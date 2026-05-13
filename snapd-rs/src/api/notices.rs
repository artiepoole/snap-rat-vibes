use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{client::SnapdClient, error::Result, types::NoticeType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Notice {
    pub id: String,
    pub user_id: Option<u64>,
    #[serde(rename = "type")]
    pub type_: NoticeType,
    pub key: String,
    pub first_occurred: String,
    pub last_occurred: String,
    pub last_repeated: Option<String>,
    pub occurrences: u64,
    #[serde(default)]
    pub last_data: HashMap<String, String>,
    pub expire_after: Option<String>,
    pub repeat_after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddNoticeResponse {
    pub id: String,
}

impl SnapdClient {
    pub async fn list_notices(&self) -> Result<Vec<Notice>> {
        self.get("/v2/notices").await
    }

    pub async fn get_notice(&self, id: &str) -> Result<Notice> {
        self.get(&format!("/v2/notices/{id}")).await
    }

    pub async fn add_notice(
        &self,
        notice_type: NoticeType,
        key: &str,
        data: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let response: AddNoticeResponse = self
            .post_sync(
                "/v2/notices",
                &json!({
                    "action": "add",
                    "type": notice_type,
                    "key": key,
                    "data": data,
                }),
            )
            .await?;
        Ok(response.id)
    }
}
