//! HTTP for the server: reads one request from a connection, checks it and
//! answers. The rules (token, Host, Origin, limits) come in the next task.

use super::Core;
use std::net::TcpStream;

pub fn serve(_core: &Core, _stream: TcpStream, _port: u16) {}
