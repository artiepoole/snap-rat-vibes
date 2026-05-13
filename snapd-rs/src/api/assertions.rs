use hyper::body::Bytes;
use serde::Deserialize;

use crate::api::model::AssertionJson;
use crate::{client::SnapdClient, error::Result};

#[derive(Debug, Deserialize)]
struct AssertionTypesResponse {
    #[serde(default)]
    types: Vec<String>,
}

impl SnapdClient {
    pub async fn list_assertion_types(&self) -> Result<Vec<String>> {
        let response: AssertionTypesResponse = self.get("/v2/assertions").await?;
        Ok(response.types)
    }

    pub async fn add_assertion(&self, assertion: &str) -> Result<()> {
        self.post_raw_sync(
            "/v2/assertions",
            Bytes::from(assertion.to_owned()),
            "text/plain",
        )
        .await
    }

    pub async fn get_assertions(&self, assert_type: &str) -> Result<Vec<AssertionJson>> {
        self.get(&format!("/v2/assertions/{assert_type}?json=true"))
            .await
    }
}
