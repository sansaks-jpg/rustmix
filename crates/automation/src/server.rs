//! Loopback HTTP listener exposing the state document and control dispatch.
//!
//! The server is deliberately minimal: HTTP/1.1 GET only, one request per
//! connection, an 8 KiB request bound, and it binds `127.0.0.1` only. It is
//! **not started automatically** by the application; an embedder decides when
//! remote control is enabled. Status codes follow the observed contract:
//! the state document answers 200, a dispatched function answers 200 (empty
//! body), an unknown function answers 500, and a well-formed request with an
//! unresolvable input target answers 200 as an accepted no-op.
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};

use rustmix_engine::Engine;

use crate::control;
use crate::document::{self, Telemetry};

/// The reference endpoint's default port.
pub const DEFAULT_PORT: u16 = 8088;
/// Requests larger than this are rejected before any parsing.
pub const MAX_REQUEST_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    fn reason(&self) -> &'static str {
        match self.status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }

    fn content_type(&self) -> &'static str {
        if self.status == 200 && self.body.starts_with("<vmix>") {
            "text/xml; charset=utf-8"
        } else {
            "text/plain; charset=utf-8"
        }
    }

    fn serialise(&self) -> Vec<u8> {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.status,
            self.reason(),
            self.content_type(),
            self.body.len(),
            self.body
        )
        .into_bytes()
    }
}

/// Handle one request line such as `GET /api/?Function=CutDirect&Input=2`.
pub fn handle_request(
    engine: &mut Engine,
    telemetry: &Telemetry,
    target: &str,
    now: u64,
) -> Response {
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path, query),
        None => (target, ""),
    };
    if path != "/api/" && path != "/api" {
        return Response {
            status: 404,
            body: "only /api/ is served".into(),
        };
    }
    if query.trim().is_empty() {
        return Response {
            status: 200,
            body: document::render(engine.project(), telemetry),
        };
    }
    let request = control::parse_query(query);
    match control::dispatch(engine, &request, now) {
        Ok(_) => Response {
            status: 200,
            body: String::new(),
        },
        Err(error) => Response {
            status: 500,
            body: error.to_string(),
        },
    }
}

/// Read one bounded HTTP request from the stream; `None` on malformed input.
pub fn read_request(stream: &mut TcpStream) -> Option<String> {
    let mut buffer = [0_u8; 1024];
    let mut request = Vec::new();
    loop {
        let read = stream.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        if request.len().saturating_add(read) > MAX_REQUEST_BYTES {
            return None;
        }
        request.extend_from_slice(buffer.get(..read).unwrap_or(&[]));
        if request.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let text = String::from_utf8_lossy(&request).into_owned();
    let request_line = text.lines().next()?;
    let mut parts = request_line.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    parts.next().map(|target| target.to_string())
}

/// Serve one connection: read, dispatch, respond, close.
pub fn serve_one(stream: &mut TcpStream, engine: &mut Engine, telemetry: &Telemetry, now: u64) {
    if !stream.peer_addr().is_ok_and(|addr| addr.ip().is_loopback()) {
        return;
    }
    if let Some(target) = read_request(stream) {
        let response = handle_request(engine, telemetry, &target, now);
        let _ = stream.write_all(&response.serialise());
        let _ = stream.flush();
    }
}

/// A bound loopback listener. The embedder owns threading and the engine; the
/// listener only accepts sequential connections until closed.
pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Bind the loopback control endpoint. Refuses non-loopback addresses.
    pub fn bind_loopback(port: u16) -> std::io::Result<Self> {
        let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), port);
        Ok(Self {
            listener: TcpListener::bind(addr)?,
        })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Accept and serve exactly one connection. Returns false when the
    /// listener produced no connection.
    pub fn accept_once(&self, engine: &mut Engine, telemetry: &Telemetry, now: u64) -> bool {
        let Ok((mut stream, _)) = self.listener.accept() else {
            return false;
        };
        serve_one(&mut stream, engine, telemetry, now);
        true
    }

    /// Accept and serve connections sequentially. Each non-loopback peer is
    /// dropped without a response.
    pub fn serve(&self, engine: &mut Engine, telemetry: &Telemetry, now: u64) {
        for stream in self.listener.incoming() {
            let Ok(mut stream) = stream else {
                continue;
            };
            serve_one(&mut stream, engine, telemetry, now);
        }
    }
}
