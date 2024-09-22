use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct ZeaburClient {
    api_key: String,
    client: Client,
}

impl ZeaburClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::new();
        ZeaburClient { api_key, client }
    }

    pub(crate) async fn execute_query(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<Value> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });

        self.client
            .post("https://gateway.zeabur.com/graphql")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .context("Failed to send request")?
            .json::<Value>()
            .await
            .context("Failed to parse response")
    }
}
