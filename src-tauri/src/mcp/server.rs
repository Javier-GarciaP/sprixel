//! The listener: binds `127.0.0.1` on a free port, writes `runtime.json`, and
//! serves connections until it is stopped. The HTTP rules are in `http.rs`.

use super::{Core, http};
use serde::Serialize;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

/// The file that tells the `sprixel` command where the server is.
pub const RUNTIME_FILE: &str = "runtime.json";

#[derive(Serialize)]
struct Runtime {
    port: u16,
    pid: u32,
}

pub fn runtime_path(data_dir: &Path) -> PathBuf {
    data_dir.join(RUNTIME_FILE)
}

pub struct Server {
    port: u16,
    addr: SocketAddr,
    data_dir: PathBuf,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Server {
    /// Starts listening. The address is always `127.0.0.1`, so no other
    /// computer can connect; the port is chosen by the system.
    pub fn start(core: Arc<Core>) -> io::Result<Server> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
        let addr = listener.local_addr()?;
        let port = addr.port();
        let data_dir = core.data_dir.clone();
        let runtime = serde_json::to_vec_pretty(&Runtime { port, pid: std::process::id() })?;
        super::token::write_private(&runtime_path(&data_dir), &runtime)?;
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let stop = stop.clone();
            std::thread::spawn(move || accept_loop(listener, core, port, stop))
        };
        Ok(Server { port, addr, data_dir, stop, thread: Some(thread) })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// The address the server listens on.
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for Server {
    /// Stops the server and removes `runtime.json`, so nobody looks for a
    /// server that is gone.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // The accept loop waits for a connection: make one to wake it up.
        let _ = TcpStream::connect((Ipv4Addr::LOCALHOST, self.port));
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        let _ = std::fs::remove_file(runtime_path(&self.data_dir));
    }
}

fn accept_loop(listener: TcpListener, core: Arc<Core>, port: u16, stop: Arc<AtomicBool>) {
    for stream in listener.incoming() {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        let Ok(stream) = stream else { continue };
        let core = core.clone();
        std::thread::spawn(move || http::serve(&core, stream, port));
    }
}
