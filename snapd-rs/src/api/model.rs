use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{client::SnapdClient, error::Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionJson {
    pub headers: HashMap<String, Value>,
    pub body: Option<String>,
}

impl SnapdClient {
    pub async fn get_model(&self) -> Result<AssertionJson> {
        self.get("/v2/model?json=true").await
    }

    pub async fn get_serial(&self) -> Result<AssertionJson> {
        self.get("/v2/model/serial?json=true").await
    }
}
