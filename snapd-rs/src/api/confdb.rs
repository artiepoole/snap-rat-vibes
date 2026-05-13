use std::collections::HashMap;

use serde_json::{Value, json};

use crate::{client::SnapdClient, error::Result, types::ChangeId};

impl SnapdClient {
    pub async fn get_confdb(&self, account: &str, schema: &str, view: &str) -> Result<ChangeId> {
        self.get_async(&format!("/v2/confdb/{account}/{schema}/{view}"))
            .await
    }

    pub async fn set_confdb(
        &self,
        account: &str,
        schema: &str,
        view: &str,
        values: HashMap<String, Value>,
    ) -> Result<ChangeId> {
        self.put_async(
            &format!("/v2/confdb/{account}/{schema}/{view}"),
            &json!({ "values": values }),
        )
        .await
    }
}
