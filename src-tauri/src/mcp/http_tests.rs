//! Task 1.3: token, Host, Origin and request limits, one test per rule, over
//! a real connection to a real server on a free port.

use super::limits::Limits;
use super::testing::TestHost;
use super::*;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

struct Answer {
    status: u16,
    headers: String,
    body: String,
}

impl Answer {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }

    fn error_code(&self) -> String {
        self.json()["error"]["code"].as_str().unwrap_or("").to_string()
    }
}

/// A tool that tells the host it ran, so tests can see that a rejected
/// request never reached a tool.
fn marker(ctx: &Ctx, _args: &Value) -> ToolResult {
    ctx.core.host.emit("tool-ran", json!(null));
    Ok(Output::Json(json!({ "ok": true })))
}

fn start_with(limits: Limits) -> (Arc<TestHost>, Arc<Core>, Server) {
    let host = Arc::new(TestHost::default());
    let tool = Tool {
        name: "marker",
        description: "Marks that it ran.",
        schema: json!({ "type": "object" }),
        need: Need::Nothing,
        run: marker,
    };
    let mut core = Core::new(host.clone(), vec![tool], host.dir.path().join(".sprixel")).unwrap();
    core.limits = limits;
    let core = Arc::new(core);
    let server = Server::start(core.clone()).unwrap();
    (host, core, server)
}

fn start() -> (Arc<TestHost>, Arc<Core>, Server) {
    start_with(Limits::default())
}

/// Sends raw bytes and reads the whole answer.
fn send(port: u16, raw: &[u8]) -> Answer {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
    stream.write_all(raw).unwrap();
    read_answer(&mut stream)
}

fn read_answer(stream: &mut TcpStream) -> Answer {
    let mut bytes = Vec::new();
    let _ = stream.read_to_end(&mut bytes);
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status = head.split(' ').nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    Answer { status, headers: head.to_string(), body: body.to_string() }
}

/// A well-formed POST with the headers in `extra` (the first `Host` wins:
/// pass `host` to change it, `None` for the right one).
fn post(port: u16, token: Option<&str>, host: Option<&str>, extra: &[&str], body: &str) -> Answer {
    let mut head = format!("POST /mcp HTTP/1.1\r\nHost: {}\r\n", host.map_or(format!("127.0.0.1:{port}"), str::to_string));
    if let Some(token) = token {
        head.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    for line in extra {
        head.push_str(&format!("{line}\r\n"));
    }
    head.push_str(&format!("Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()));
    send(port, head.as_bytes())
}

fn initialize() -> String {
    json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "clientInfo": { "name": "Claude Code", "version": "1" } } })
    .to_string()
}

fn tool_ran(host: &TestHost) -> bool {
    host.events.lock().unwrap().iter().any(|(event, _)| event == "tool-ran")
}

fn header<'a>(answer: &'a Answer, name: &str) -> Option<&'a str> {
    answer.headers.lines().find_map(|line| {
        let (n, v) = line.split_once(':')?;
        n.eq_ignore_ascii_case(name).then(|| v.trim())
    })
}

// Checks the "Missing token" scenario: no token gives "unauthorized".
#[test]
fn a_request_without_a_token_is_unauthorized() {
    let (_host, _core, server) = start();
    let answer = post(server.port(), None, None, &[], &initialize());
    assert_eq!(answer.status, 401);
    assert_eq!(answer.error_code(), "unauthorized");
}

// Checks that a wrong token, a token with a different length and a wrong
// scheme are all unauthorized, and that none of them gets data or a session.
#[test]
fn a_wrong_token_is_unauthorized() {
    let (_host, core, server) = start();
    let port = server.port();
    // Same length, last character changed (`x` is never in a hex token).
    let almost = format!("{}x", &core.token[..core.token.len() - 1]);
    for token in ["wrong", almost.as_str(), &core.token[1..]] {
        let answer = post(port, Some(token), None, &[], &initialize());
        assert_eq!(answer.status, 401, "{token}");
        assert!(header(&answer, "mcp-session-id").is_none());
    }
    let basic = post(port, None, None, &[&format!("Authorization: Basic {}", core.token)], &initialize());
    assert_eq!(basic.status, 401);
}

// Checks the happy path over HTTP: a valid token starts a session, and the
// session lists the tools and calls one. No CORS header is ever sent.
#[test]
fn a_valid_token_can_list_and_call_tools() {
    let (host, core, server) = start();
    let port = server.port();
    let first = post(port, Some(&core.token), None, &[], &initialize());
    assert_eq!(first.status, 200);
    let session = header(&first, "mcp-session-id").expect("a session id").to_string();
    let session_header = format!("Mcp-Session-Id: {session}");

    let list = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }).to_string();
    let listed = post(port, Some(&core.token), None, &[&session_header], &list);
    assert_eq!(listed.json()["result"]["tools"][0]["name"], "marker");

    let call = json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": { "name": "marker" } }).to_string();
    let called = post(port, Some(&core.token), None, &[&session_header], &call);
    assert_eq!(called.json()["result"]["isError"], false);
    assert!(tool_ran(&host));
    assert!(!first.headers.to_ascii_lowercase().contains("access-control"));
    assert!(!called.headers.to_ascii_lowercase().contains("access-control"));
}

// Checks the "Request from a web page" scenario: an Origin header is
// rejected even with a valid token, for any value, and no tool runs.
#[test]
fn a_request_with_an_origin_is_rejected_even_with_a_valid_token() {
    let (host, core, server) = start();
    for origin in ["https://evil.example", "null", "http://127.0.0.1:1420", ""] {
        let line = format!("Origin: {origin}");
        let call = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "marker" } }).to_string();
        let answer = post(server.port(), Some(&core.token), None, &[&line], &call);
        assert_eq!(answer.status, 403, "{origin:?}");
        assert_eq!(answer.error_code(), "forbidden");
    }
    assert!(!tool_ran(&host));
}

// Checks the Host rule (DNS rebinding): only our own address and port, even
// with a valid token. A name like `evil.example`, the wrong port, no Host at
// all and a Host with extra text are refused; `localhost` is accepted.
#[test]
fn a_wrong_host_is_rejected_and_localhost_is_accepted() {
    let (_host, core, server) = start();
    let port = server.port();
    let evil = "evil.example";
    let other_port = format!("127.0.0.1:{}", port.wrapping_add(1));
    let suffix = format!("127.0.0.1:{port}.evil.example");
    for host in [evil, other_port.as_str(), "127.0.0.1", suffix.as_str(), "0.0.0.0"] {
        let answer = post(port, Some(&core.token), Some(host), &[], &initialize());
        assert_eq!(answer.status, 403, "{host}");
    }
    // No Host header at all (HTTP/1.0 style).
    let body = initialize();
    let raw = format!(
        "POST /mcp HTTP/1.0\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\n\r\n{body}",
        core.token,
        body.len()
    );
    assert_eq!(send(port, raw.as_bytes()).status, 403);

    let local = post(port, Some(&core.token), Some(&format!("localhost:{port}")), &[], &initialize());
    assert_eq!(local.status, 200);
}

// Checks the "Huge request" scenario: a body over 4 MB is refused with
// `limit_exceeded` before it is read, and the server keeps working.
#[test]
fn a_huge_request_is_refused_and_the_server_keeps_working() {
    let (_host, core, server) = start();
    let port = server.port();
    let size = 4 * 1024 * 1024 + 1;
    let head = format!(
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {}\r\nContent-Length: {size}\r\n\r\n",
        core.token
    );
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
    stream.write_all(head.as_bytes()).unwrap();
    // A part of the body: the server must not wait for all of it.
    let _ = stream.write_all(&vec![b'x'; 64 * 1024]);
    let answer = read_answer(&mut stream);
    assert_eq!(answer.status, 413);
    assert_eq!(answer.error_code(), "limit_exceeded");

    assert_eq!(post(port, Some(&core.token), None, &[], &initialize()).status, 200);
}

// Checks that a body of exactly 4 MB is not refused by the size rule (it is
// not JSON, so it fails later with a protocol error, not with 413).
#[test]
fn a_body_of_the_maximum_size_is_accepted_by_the_size_rule() {
    let (_host, core, server) = start();
    let body = " ".repeat(4 * 1024 * 1024);
    let answer = post(server.port(), Some(&core.token), None, &[], &body);
    assert_eq!(answer.status, 200);
    assert_eq!(answer.json()["error"]["code"], -32700);
}

// Checks the time limit: a client that sends half of the headers and stops
// gets `408` and the connection is closed, instead of waiting forever.
#[test]
fn a_slow_request_times_out() {
    let limits = Limits { request_time: Duration::from_millis(300), ..Limits::default() };
    let (_host, _core, server) = start_with(limits);
    let mut stream = TcpStream::connect(("127.0.0.1", server.port())).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
    stream.write_all(b"POST /mcp HTTP/1.1\r\nHost: 127.0.0.1").unwrap();

    let answer = read_answer(&mut stream);

    assert_eq!(answer.status, 408);
    assert_eq!(answer.error_code(), "timeout");
}

// Checks the time limit for a body: the headers are good, the body stops.
#[test]
fn a_body_that_stops_times_out() {
    let limits = Limits { request_time: Duration::from_millis(300), ..Limits::default() };
    let (_host, core, server) = start_with(limits);
    let port = server.port();
    let head = format!(
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {}\r\nContent-Length: 100\r\n\r\n{{\"jsonrpc\"",
        core.token
    );
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
    stream.write_all(head.as_bytes()).unwrap();

    assert_eq!(read_answer(&mut stream).status, 408);
}

// Checks the rate limit: after N requests in one second the next one gets
// `limit_exceeded` (429).
#[test]
fn too_many_requests_per_second_are_refused() {
    let limits = Limits { requests_per_second: 3, ..Limits::default() };
    let (_host, core, server) = start_with(limits);
    let statuses: Vec<u16> = (0..5)
        .map(|_| post(server.port(), Some(&core.token), None, &[], &initialize()).status)
        .collect();
    assert_eq!(statuses[..3], [200, 200, 200]);
    assert_eq!(post(server.port(), Some(&core.token), None, &[], &initialize()).error_code(), "limit_exceeded");
    assert!(statuses[3..].iter().all(|s| *s == 429), "{statuses:?}");
}

// Checks the limit of connections open at the same time: a connection that
// stays open uses the only place, and the next one gets `limit_exceeded`.
#[test]
fn too_many_connections_are_refused() {
    let limits = Limits { connections: 1, request_time: Duration::from_secs(5), ..Limits::default() };
    let (_host, core, server) = start_with(limits);
    let port = server.port();
    let mut stalled = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stalled.write_all(b"POST /mcp HTT").unwrap();
    // The server counts the connection when it starts serving it: wait for it.
    let start = std::time::Instant::now();
    while core.connections.load(std::sync::atomic::Ordering::SeqCst) == 0 {
        assert!(start.elapsed() < Duration::from_secs(5), "timed out");
        std::thread::sleep(Duration::from_millis(5));
    }

    let answer = post(port, Some(&core.token), None, &[], &initialize());

    assert_eq!(answer.status, 503);
    assert_eq!(answer.error_code(), "limit_exceeded");
    drop(stalled);
}

// Checks the "Private logs" requirement: a failed authorization is logged
// with its reason, and neither the token that was sent nor the real token is
// in the log.
#[test]
fn the_log_has_the_reason_but_not_the_token() {
    let (_host, core, server) = start();
    let sent = "sent-token-1234567890abcdef";
    post(server.port(), Some(sent), None, &[], &initialize());
    post(server.port(), Some(&core.token), None, &["Origin: https://evil.example"], &initialize());

    let logs = core.logs.lock().unwrap().iter().cloned().collect::<Vec<_>>().join("\n");

    assert!(logs.contains("missing or wrong token"), "{logs}");
    assert!(logs.contains("Origin"), "{logs}");
    assert!(!logs.contains(sent));
    assert!(!logs.contains(&core.token));
    assert!(!logs.contains("evil.example"));
}

// Checks that requests that could confuse a parser are refused: chunked
// bodies, a repeated Host or Content-Length, a folded header, a bad length
// and a missing length.
#[test]
fn ambiguous_requests_are_refused() {
    let (_host, core, server) = start();
    let port = server.port();
    let t = &core.token;
    let host = format!("Host: 127.0.0.1:{port}");
    let cases: Vec<(String, u16)> = vec![
        (format!("POST /mcp HTTP/1.1\r\n{host}\r\nAuthorization: Bearer {t}\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n"), 400),
        (format!("POST /mcp HTTP/1.1\r\n{host}\r\nHost: evil.example\r\nAuthorization: Bearer {t}\r\nContent-Length: 0\r\n\r\n"), 403),
        (format!("POST /mcp HTTP/1.1\r\n{host}\r\nAuthorization: Bearer {t}\r\nContent-Length: 2\r\nContent-Length: 3\r\n\r\n{{}}"), 400),
        (format!("POST /mcp HTTP/1.1\r\n{host}\r\nAuthorization: Bearer {t}\r\n X-Folded: yes\r\nContent-Length: 0\r\n\r\n"), 400),
        (format!("POST /mcp HTTP/1.1\r\n{host}\r\nAuthorization: Bearer {t}\r\nContent-Length: +5\r\n\r\n12345"), 400),
        (format!("POST /mcp HTTP/1.1\r\n{host}\r\nAuthorization: Bearer {t}\r\n\r\n"), 411),
        ("not http at all\r\n\r\n".to_string(), 400),
    ];
    for (raw, status) in cases {
        assert_eq!(send(port, raw.as_bytes()).status, status, "{raw:?}");
    }
}

// Checks the path and the method: only `POST /mcp` runs tools. Other paths
// are 404 and other methods 405, and only for a client with the token.
#[test]
fn only_post_to_the_mcp_path_works() {
    let (_host, core, server) = start();
    let port = server.port();
    let get = format!("GET /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {}\r\n\r\n", core.token);
    assert_eq!(send(port, get.as_bytes()).status, 405);
    let other = format!("POST /admin HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {}\r\nContent-Length: 0\r\n\r\n", core.token);
    assert_eq!(send(port, other.as_bytes()).status, 404);
    let anonymous = format!("GET /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n");
    assert_eq!(send(port, anonymous.as_bytes()).status, 401);
}

// Checks that the client can end its session with DELETE, and that the
// session no longer works after it.
#[test]
fn a_session_can_be_ended() {
    let (_host, core, server) = start();
    let port = server.port();
    let first = post(port, Some(&core.token), None, &[], &initialize());
    let session = header(&first, "mcp-session-id").unwrap().to_string();
    let delete = format!(
        "DELETE /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {}\r\nMcp-Session-Id: {session}\r\n\r\n",
        core.token
    );
    assert_eq!(send(port, delete.as_bytes()).status, 200);

    let list = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }).to_string();
    let answer = post(port, Some(&core.token), None, &[&format!("Mcp-Session-Id: {session}")], &list);
    assert_eq!(answer.status, 400);
}
