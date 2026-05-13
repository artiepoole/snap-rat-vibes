use std::path::{Path, PathBuf};

use http_body_util::{BodyExt, Full};
use hyper::{
    Method, Request,
    body::Bytes,
    header::{CONTENT_TYPE, HOST},
};
use hyper_util::rt::TokioIo;
use serde::{Serialize, de::DeserializeOwned};
use tokio::net::UnixStream;

use crate::{
    error::{Error, Result},
    types::{ChangeId, ErrorResult, RawSnapdResponse, ResponseType},
};

/// How the client connects to the snapd daemon.
#[derive(Debug, Clone)]
pub enum SocketAddress {
    /// A filesystem path to a Unix socket (e.g. `/run/snapd.socket`).
    Filesystem(PathBuf),
    /// An abstract Unix socket name (without the leading null byte).
    /// Used when connecting from inside snap confinement.
    Abstract(String),
}

/// The default socket used by unconfined host processes.
pub const SNAPD_SOCKET: &str = "/run/snapd.socket";

/// The socket used by snaps connecting to snapd (less-privileged).
/// This is a filesystem socket that snapd listens on for confined snap clients.
pub const SNAPD_SNAP_SOCKET: &str = "/run/snapd-snap.socket";

pub struct SnapdClient {
    socket: SocketAddress,
}

impl SnapdClient {
    /// Create a client that connects via the default snapd socket
    /// (`/run/snapd.socket`) for unconfined host processes.
    pub fn new() -> Self {
        Self {
            socket: SocketAddress::Filesystem(PathBuf::from(SNAPD_SOCKET)),
        }
    }

    /// Create a client that connects via the less-privileged snap socket
    /// (`/run/snapd-snap.socket`) used by confined snaps communicating
    /// with snapd over a filesystem path.
    pub fn new_for_snap() -> Self {
        Self {
            socket: SocketAddress::Filesystem(PathBuf::from(SNAPD_SNAP_SOCKET)),
        }
    }

    /// Create a client that connects via an abstract Unix socket.
    /// The name should not include the leading null byte — it is added
    /// automatically. Use this when connecting from within snap
    /// confinement where the filesystem socket is not accessible.
    pub fn new_abstract(name: impl Into<String>) -> Self {
        Self {
            socket: SocketAddress::Abstract(name.into()),
        }
    }

    /// Create a client with an explicit filesystem socket path.
    pub fn with_socket(path: impl AsRef<Path>) -> Self {
        Self {
            socket: SocketAddress::Filesystem(path.as_ref().to_path_buf()),
        }
    }

    async fn connect(&self) -> Result<UnixStream> {
        match &self.socket {
            SocketAddress::Filesystem(path) => Ok(UnixStream::connect(path).await?),
            SocketAddress::Abstract(name) => {
                use std::os::linux::net::SocketAddrExt;
                let addr = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes())
                    .map_err(|e| Error::Connection(format!("invalid abstract socket name: {e}")))?;
                let std_stream =
                    std::os::unix::net::UnixStream::connect_addr(&addr).map_err(|e| {
                        Error::Connection(format!(
                            "failed to connect to abstract socket @{name}: {e}"
                        ))
                    })?;
                std_stream.set_nonblocking(true)?;
                Ok(UnixStream::from_std(std_stream)?)
            }
        }
    }

    async fn send_request(
        &self,
        req: Request<Full<Bytes>>,
    ) -> Result<hyper::Response<hyper::body::Incoming>> {
        let stream = self.connect().await?;
        let io = TokioIo::new(stream);
        let (mut sender, connection) = hyper::client::conn::http1::handshake(io).await?;

        tokio::spawn(async move {
            let _ = connection.await;
        });

        Ok(sender.send_request(req).await?)
    }

    fn build_request(
        &self,
        method: Method,
        path: &str,
        body: Bytes,
        content_type: Option<&str>,
    ) -> Result<Request<Full<Bytes>>> {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header(HOST, "localhost");

        if let Some(ct) = content_type {
            builder = builder.header(CONTENT_TYPE, ct);
        }

        Ok(builder.body(Full::new(body))?)
    }

    pub(crate) async fn get_bytes(&self, path: &str) -> Result<Bytes> {
        let req = self.build_request(Method::GET, path, Bytes::new(), None)?;
        let response = self.send_request(req).await?;
        Ok(response.into_body().collect().await?.to_bytes())
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let req = self.build_request(Method::GET, path, Bytes::new(), None)?;
        self.execute_sync(req).await
    }

    pub(crate) async fn get_async(&self, path: &str) -> Result<ChangeId> {
        let req = self.build_request(Method::GET, path, Bytes::new(), None)?;
        self.execute_async(req).await
    }

    pub(crate) async fn post_sync<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let payload = serde_json::to_vec(body)?;
        let req = self.build_request(
            Method::POST,
            path,
            Bytes::from(payload),
            Some("application/json"),
        )?;
        self.execute_sync(req).await
    }

    pub(crate) async fn post_async<B: Serialize>(&self, path: &str, body: &B) -> Result<ChangeId> {
        let payload = serde_json::to_vec(body)?;
        let req = self.build_request(
            Method::POST,
            path,
            Bytes::from(payload),
            Some("application/json"),
        )?;
        self.execute_async(req).await
    }

    pub(crate) async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let payload = serde_json::to_vec(body)?;
        let req = self.build_request(
            Method::PUT,
            path,
            Bytes::from(payload),
            Some("application/json"),
        )?;
        self.execute_sync(req).await
    }

    pub(crate) async fn put_async<B: Serialize>(&self, path: &str, body: &B) -> Result<ChangeId> {
        let payload = serde_json::to_vec(body)?;
        let req = self.build_request(
            Method::PUT,
            path,
            Bytes::from(payload),
            Some("application/json"),
        )?;
        self.execute_async(req).await
    }

    pub(crate) async fn post_raw_sync<T: DeserializeOwned>(
        &self,
        path: &str,
        body: Bytes,
        content_type: &'static str,
    ) -> Result<T> {
        let req = self.build_request(Method::POST, path, body, Some(content_type))?;
        self.execute_sync(req).await
    }

    async fn execute_sync<T: DeserializeOwned>(&self, req: Request<Full<Bytes>>) -> Result<T> {
        let response = self.send_request(req).await?;
        let body = response.into_body().collect().await?.to_bytes();
        let envelope: RawSnapdResponse = serde_json::from_slice(&body)?;

        match envelope.response_type {
            ResponseType::Sync => Ok(serde_json::from_value(
                envelope.result.unwrap_or(serde_json::Value::Null),
            )?),
            ResponseType::Error => Err(self.parse_error(&envelope)),
            ResponseType::Async => Err(Error::UnexpectedResponseType("async".to_string())),
        }
    }

    async fn execute_async(&self, req: Request<Full<Bytes>>) -> Result<ChangeId> {
        let response = self.send_request(req).await?;
        let body = response.into_body().collect().await?.to_bytes();
        let envelope: RawSnapdResponse = serde_json::from_slice(&body)?;

        match envelope.response_type {
            ResponseType::Async => match envelope.change {
                Some(change) => Ok(ChangeId(change)),
                None => Err(Error::UnexpectedResponseType(
                    "async response missing change id".to_string(),
                )),
            },
            ResponseType::Error => Err(self.parse_error(&envelope)),
            ResponseType::Sync => Err(Error::UnexpectedResponseType("sync".to_string())),
        }
    }

    fn parse_error(&self, envelope: &RawSnapdResponse) -> Error {
        let error = match &envelope.result {
            Some(value) => {
                serde_json::from_value::<ErrorResult>(value.clone()).unwrap_or_else(|_| {
                    ErrorResult {
                        message: envelope.status.clone(),
                        kind: None,
                    }
                })
            }
            None => ErrorResult {
                message: envelope.status.clone(),
                kind: None,
            },
        };
        Error::Snapd {
            kind: error.kind.unwrap_or_else(|| "unknown".to_string()),
            message: error.message,
        }
    }
}

impl Default for SnapdClient {
    fn default() -> Self {
        Self::new()
    }
}
