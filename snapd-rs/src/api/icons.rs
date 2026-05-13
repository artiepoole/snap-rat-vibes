use hyper::body::Bytes;

use crate::{client::SnapdClient, error::Result};

impl SnapdClient {
    /// Fetch the icon for an installed snap from `/v2/icons/<name>/icon`.
    /// Returns raw image bytes (PNG, SVG, JPEG, …).
    pub async fn get_snap_icon(&self, name: &str) -> Result<Bytes> {
        self.get_bytes(&format!("/v2/icons/{name}/icon")).await
    }
}
