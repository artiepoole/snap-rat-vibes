//! Mock snapd server for testing
//!
//! Provides a minimal HTTP server over Unix socket that mimics snapd responses.

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

static MOCK_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Mock snapd server that listens on a Unix socket
pub struct MockSnapd {
    socket_path: PathBuf,
    thread_handle: Option<thread::JoinHandle<()>>,
    shutdown: Arc<Mutex<bool>>,
}

impl MockSnapd {
    /// Start a new mock snapd server on a unique socket path
    pub fn start() -> std::io::Result<Self> {
        let counter = MOCK_COUNTER.fetch_add(1, Ordering::SeqCst);
        let socket_path = env::temp_dir().join(format!(
            "mock-snapd-test-{}-{}.socket",
            std::process::id(),
            counter
        ));

        // Clean up any leftover socket
        let _ = fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path)?;
        println!("Mock snapd listening on {}", socket_path.display());

        let shutdown = Arc::new(Mutex::new(false));
        let shutdown_clone = Arc::clone(&shutdown);

        // Spawn server thread
        let thread_handle = thread::spawn(move || handle_connections(&listener, &shutdown_clone));

        Ok(MockSnapd {
            socket_path,
            thread_handle: Some(thread_handle),
            shutdown,
        })
    }

    /// Get the socket path for this mock server
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

impl Drop for MockSnapd {
    fn drop(&mut self) {
        // Signal shutdown
        *self.shutdown.lock().unwrap() = true;

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        // Clean up socket file
        let _ = fs::remove_file(&self.socket_path);
        println!("Mock snapd stopped");
    }
}

/// Handle incoming connections
fn handle_connections(listener: &UnixListener, shutdown: &Arc<Mutex<bool>>) {
    listener.set_nonblocking(true).ok();

    loop {
        if *shutdown.lock().unwrap() {
            break;
        }

        match listener.accept() {
            Ok((socket, _)) => {
                handle_client(socket);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }
}

/// Handle a single client connection
fn handle_client(mut socket: UnixStream) {
    let mut buffer = vec![0; 4096];

    if let Ok(n) = socket.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer[..n]);

        // Parse HTTP request line
        let response = if request.starts_with("GET /v2/system-info") {
            handle_system_info()
        } else if request.starts_with("GET /v2/apps") {
            handle_list_apps()
        } else if request.starts_with("GET /v2/snaps") {
            handle_list_snaps()
        } else {
            http_response_404()
        };

        let _ = socket.write_all(response.as_bytes());
        let _ = socket.shutdown(Shutdown::Write);
    }
}

/// HTTP response for GET /v2/system-info
fn handle_system_info() -> String {
    let body = r#"{"type":"sync","status-code":200,"status":"OK","result":{"architecture":"x86_64","build-id":"","kernel-version":"6.8.0-1011-generic","managed":false,"on-classic":true,"series":"24","system-mode":null,"version":"2.75.2"}}"#;
    http_response_ok(body)
}

/// HTTP response for GET /v2/apps
fn handle_list_apps() -> String {
    let body = r#"{"type":"sync","status-code":200,"status":"OK","result":[]}"#;
    http_response_ok(body)
}

/// HTTP response for GET /v2/snaps
fn handle_list_snaps() -> String {
    let body = r#"{"type":"sync","status-code":200,"status":"OK","result":[{"name":"test-snap","version":"1.0","status":"active","type":"app","channel":"","tracking-channel":"","revision":1}]}"#;
    http_response_ok(body)
}

/// Build an HTTP 200 response
fn http_response_ok(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

/// Build an HTTP 404 response
fn http_response_404() -> String {
    let body = r#"{"type":"error","status-code":404,"status":"Not Found","result":{"message":"Not Found"}}"#;
    format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}
