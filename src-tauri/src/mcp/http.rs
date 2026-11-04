//! HTTP for the server: reads one request from a connection, checks it and
//! answers. One request per connection (`Connection: close`) and only
//! `Content-Length` bodies, which keeps the parser small and safe.
//!
//! The checks, in this order, before the body is read: `Host` (so a web page
//! cannot reach us through DNS rebinding), `Origin` (a browser always sends
//! it), the token, the path and method, the size and the rate. Each answers
//! with a stable error code and never repeats what the client sent.

use super::{Core, token};
use serde_json::{Value, json};
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

/// The only path of the server.
pub const PATH: &str = "/mcp";
/// The headers of a request must fit here.
const MAX_HEADERS: usize = 16 * 1024;
/// The header that carries the session id (MCP).
const SESSION_HEADER: &str = "mcp-session-id";

struct Response {
    status: u16,
    session: Option<String>,
    body: Option<Value>,
}

fn failure(status: u16, code: &str, message: &str) -> Response {
    Response {
        status,
        session: None,
        body: Some(json!({ "error": { "code": code, "message": message } })),
    }
}

/// Keeps count of the connections being served.
struct Connection<'a>(&'a Core);

impl<'a> Connection<'a> {
    fn enter(core: &'a Core) -> Option<Connection<'a>> {
        if core.connections.fetch_add(1, Ordering::SeqCst) >= core.limits.connections {
            core.connections.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        Some(Connection(core))
    }
}

impl Drop for Connection<'_> {
    fn drop(&mut self) {
        self.0.connections.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Serves one connection: one request and one answer.
pub fn serve(core: &Core, mut stream: TcpStream, port: u16) {
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let response = match Connection::enter(core) {
        Some(_connection) => {
            let deadline = Instant::now() + core.limits.request_time;
            process(core, &mut stream, port, deadline)
        }
        None => failure(503, "limit_exceeded", "too many connections, try again"),
    };
    let _ = write_response(&mut stream, &response);
    // Say goodbye in order, then read what is left of the request, so the
    // client gets the answer instead of a reset connection.
    let _ = stream.shutdown(Shutdown::Write);
    drain(&mut stream);
}

fn process(core: &Core, stream: &mut TcpStream, port: u16, deadline: Instant) -> Response {
    let head = match read_head(stream, deadline) {
        Ok(head) => head,
        Err(response) => return response,
    };
    let request = match Request::parse(&head.text) {
        Ok(request) => request,
        Err(message) => return failure(400, "bad_request", message),
    };

    // 1. Host: only our own address and port.
    let host_ok = request.header("host").ok().flatten().is_some_and(|host| {
        host == format!("127.0.0.1:{port}") || host.eq_ignore_ascii_case(&format!("localhost:{port}"))
    });
    if !host_ok {
        core.log("request rejected: the Host header is not the local address");
        return failure(403, "forbidden", "the Host header must be the local address and port");
    }
    // 2. Origin: a request from a web page has it. Ours never do.
    if request.has_header("origin") {
        core.log("request rejected: it has an Origin header");
        return failure(403, "forbidden", "requests from web pages are not allowed");
    }
    // 3. Token.
    let token_ok = match request.header("authorization") {
        Ok(Some(value)) => bearer(value).is_some_and(|sent| token::same(sent, &core.token)),
        _ => false,
    };
    if !token_ok {
        core.log("request rejected: missing or wrong token");
        return failure(401, "unauthorized", "send the access token as `Authorization: Bearer <token>`");
    }
    // 4. Path and method.
    if request.path != PATH {
        return failure(404, "not_found", "the only path is /mcp");
    }
    let session = match request.header(SESSION_HEADER) {
        Ok(session) => session,
        Err(()) => return failure(400, "bad_request", "the session header is repeated"),
    };
    // 5. Rate: counted for the session, or for all the requests without a
    // known session.
    let key = session.filter(|id| core.sessions.lock().unwrap().exists(id)).unwrap_or("-");
    if !core.rate.lock().unwrap().allow(key, core.limits.requests_per_second) {
        core.log("request rejected: too many requests");
        return failure(429, "limit_exceeded", "too many requests per second, slow down");
    }
    match request.method.as_str() {
        "DELETE" => {
            if let Some(id) = session {
                core.sessions.lock().unwrap().end(id);
            }
            return Response { status: 200, session: None, body: None };
        }
        "POST" => {}
        _ => return failure(405, "method_not_allowed", "use POST"),
    }
    // 6. Size.
    let length = match request.content_length() {
        Ok(Some(length)) => length,
        Ok(None) => return failure(411, "length_required", "send a Content-Length header"),
        Err(message) => return failure(400, "bad_request", message),
    };
    if length > core.limits.body {
        core.log("request rejected: the body is too large");
        return failure(413, "limit_exceeded", "the request is larger than 4 MB");
    }
    // 7. The body, inside the time limit.
    let body = match read_body(stream, head.rest, length, deadline) {
        Ok(body) => body,
        Err(response) => return response,
    };
    let reply = core.handle(session, &body);
    Response { status: reply.status, session: reply.session, body: reply.body }
}

fn bearer(value: &str) -> Option<&str> {
    let (scheme, token) = value.split_once(' ')?;
    scheme.eq_ignore_ascii_case("bearer").then_some(token.trim())
}

/// The request line and the headers.
struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
}

impl Request {
    fn parse(head: &str) -> Result<Request, &'static str> {
        let mut lines = head.split("\r\n");
        let mut first = lines.next().unwrap_or("").split(' ');
        let (Some(method), Some(target), Some(version), None) =
            (first.next(), first.next(), first.next(), first.next())
        else {
            return Err("the request line is not valid");
        };
        if version != "HTTP/1.1" && version != "HTTP/1.0" {
            return Err("only HTTP/1.0 and HTTP/1.1 are supported");
        }
        let mut headers = Vec::new();
        for line in lines.filter(|line| !line.is_empty()) {
            // A line that starts with a space continues the line before. That
            // old form is used to confuse parsers, so it is refused.
            let Some((name, value)) = line.split_once(':').filter(|(n, _)| {
                !n.is_empty() && n.bytes().all(|b| b.is_ascii_graphic() && b != b':')
            }) else {
                return Err("a header is not valid");
            };
            headers.push((name.to_ascii_lowercase(), value.trim().to_string()));
        }
        let request = Request {
            method: method.to_string(),
            // The query is not used: `/mcp?x` is not `/mcp`.
            path: target.to_string(),
            headers,
        };
        // A body in pieces is not supported; refusing it removes the classic
        // confusion between Content-Length and Transfer-Encoding.
        if request.has_header("transfer-encoding") {
            return Err("Transfer-Encoding is not supported, send a Content-Length");
        }
        Ok(request)
    }

    fn has_header(&self, name: &str) -> bool {
        self.headers.iter().any(|(n, _)| n == name)
    }

    /// The value of a header: `Ok(None)` when absent, `Err` when repeated.
    fn header(&self, name: &str) -> Result<Option<&str>, ()> {
        let mut found = self.headers.iter().filter(|(n, _)| n == name);
        match (found.next(), found.next()) {
            (None, _) => Ok(None),
            (Some((_, value)), None) => Ok(Some(value)),
            _ => Err(()),
        }
    }

    fn content_length(&self) -> Result<Option<usize>, &'static str> {
        match self.header("content-length") {
            Ok(None) => Ok(None),
            Ok(Some(value)) if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) => {
                value.parse().map(Some).map_err(|_| "Content-Length is too large")
            }
            _ => Err("Content-Length is not valid"),
        }
    }
}

struct Head {
    text: String,
    /// The bytes of the body that came with the headers.
    rest: Vec<u8>,
}

fn read_head(stream: &mut TcpStream, deadline: Instant) -> Result<Head, Response> {
    let mut buffer = Vec::new();
    loop {
        if let Some(end) = find(&buffer, b"\r\n\r\n") {
            let text = String::from_utf8(buffer[..end].to_vec())
                .map_err(|_| failure(400, "bad_request", "the headers are not text"))?;
            return Ok(Head { text, rest: buffer[end + 4..].to_vec() });
        }
        if buffer.len() > MAX_HEADERS {
            return Err(failure(431, "limit_exceeded", "the headers are too large"));
        }
        read_more(stream, &mut buffer, deadline)?;
    }
}

fn read_body(
    stream: &mut TcpStream,
    mut body: Vec<u8>,
    length: usize,
    deadline: Instant,
) -> Result<Vec<u8>, Response> {
    if body.len() > length {
        return Err(failure(400, "bad_request", "the body is longer than Content-Length"));
    }
    while body.len() < length {
        read_more(stream, &mut body, deadline)?;
        if body.len() > length {
            return Err(failure(400, "bad_request", "the body is longer than Content-Length"));
        }
    }
    Ok(body)
}

/// Reads some bytes, but never past the deadline of the whole request.
fn read_more(stream: &mut TcpStream, buffer: &mut Vec<u8>, deadline: Instant) -> Result<(), Response> {
    let timeout = || failure(408, "timeout", "the request took too long to arrive");
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return Err(timeout());
    }
    let _ = stream.set_read_timeout(Some(left));
    let mut chunk = [0u8; 8192];
    match stream.read(&mut chunk) {
        Ok(0) => Err(failure(400, "bad_request", "the request ended too soon")),
        Ok(n) => {
            buffer.extend_from_slice(&chunk[..n]);
            Ok(())
        }
        Err(e) if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => Err(timeout()),
        Err(_) => Err(failure(400, "bad_request", "the request could not be read")),
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn write_response(stream: &mut TcpStream, response: &Response) -> io::Result<()> {
    let body = response.body.as_ref().map(Value::to_string).unwrap_or_default();
    let mut head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n",
        response.status,
        reason(response.status),
        body.len()
    );
    // No CORS headers, ever: a web page must not be able to read an answer.
    if let Some(session) = &response.session {
        head.push_str(&format!("Mcp-Session-Id: {session}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    stream.flush()
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        411 => "Length Required",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

/// Reads and throws away what the client is still sending, for a short time
/// and a small size, so a big upload we refused does not reset the answer.
fn drain(stream: &mut TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
    let mut left: usize = 1024 * 1024;
    let mut chunk = [0u8; 8192];
    while left > 0 {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => left = left.saturating_sub(n),
        }
    }
}
