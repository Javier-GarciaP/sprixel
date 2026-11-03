//! Task 1.2: the loopback server, the free port, `runtime.json` and the token.

use super::server::runtime_path;
use super::testing::TestHost;
use super::*;
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::sync::Arc;

fn start() -> (Arc<TestHost>, Arc<Core>, Server) {
    let host = Arc::new(TestHost::default());
    let core = Arc::new(Core::new(host.clone(), vec![], host.dir.path().join(".sprixel")).unwrap());
    let server = Server::start(core.clone()).unwrap();
    (host, core, server)
}

// Checks the "Local only" requirement: the server listens on 127.0.0.1, not
// on 0.0.0.0, so it is not reachable from the network. We check the address
// the system gave to the socket, which is what decides who can connect.
#[test]
fn the_server_listens_on_loopback_only() {
    let (_host, _core, server) = start();
    // The server answers on loopback.
    assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, server.port())).is_ok());
    // And the address it reports is loopback, never the wildcard.
    let addr = server.local_addr();
    assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert!(!addr.ip().is_unspecified());
}

// Checks that the port is chosen by the system (two servers get two ports)
// and that `runtime.json` has that port and the process id.
#[test]
fn the_port_is_free_and_written_to_runtime_json() {
    let (host, _core, server) = start();
    let text = std::fs::read_to_string(runtime_path(&host.dir.path().join(".sprixel"))).unwrap();
    let runtime: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(runtime["port"], server.port());
    assert_eq!(runtime["pid"], std::process::id());
    assert_ne!(server.port(), 0);

    let (_h2, _c2, second) = start();
    assert_ne!(server.port(), second.port());
}

// Checks that the server stops with the app: the port is closed and
// `runtime.json` is removed.
#[test]
fn stopping_removes_runtime_json_and_closes_the_port() {
    let (host, _core, server) = start();
    let port = server.port();
    let runtime = runtime_path(&host.dir.path().join(".sprixel"));
    assert!(runtime.is_file());

    drop(server);

    assert!(!runtime.exists());
    assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, port)).is_err());
}

// Checks that the token of the core is the one in the token file, and that
// the token file is private to the user (mode 0600 on Linux and macOS).
#[test]
fn the_core_uses_the_token_file_and_it_is_private() {
    let (host, core, _server) = start();
    let path = token::token_path(&host.dir.path().join(".sprixel"));
    assert_eq!(std::fs::read_to_string(&path).unwrap().trim(), core.token);
    assert!(core.token.len() >= 32);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }
}
