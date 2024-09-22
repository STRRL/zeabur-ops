use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::ZeaburClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeLog {
    pub timestamp: String,
    pub message: String,
    #[serde(rename = "zeaburUID")]
    pub zeabur_uid: String,
}

impl ZeaburClient {
    pub async fn query_service_runtime_logs(
        &self,
        project_id: &str,
        service_id: &str,
        environment_id: &str,
        timestamp_cursor: Option<&str>,
    ) -> Result<Vec<RuntimeLog>> {
        let query = format!(
            r#"
            query {{
                runtimeLogs(
                    projectID: "{project_id}",
                    environmentID: "{environment_id}",
                    serviceID: "{service_id}",
                    {timestamp_cursor_param}
                ) {{
                    timestamp
                    message
                    zeaburUID
                }}
            }}
            "#,
            timestamp_cursor_param = timestamp_cursor
                .map(|ts| format!("timestampCursor: \"{ts}\""))
                .unwrap_or_default()
        );

        let variables = serde_json::json!({
            "projectID": project_id,
            "serviceID": service_id,
            "environmentID": environment_id,
            "timestampCursor": timestamp_cursor
        });

        let response = self.execute_query(&query, variables).await?;
        self.parse_runtime_logs(response)
    }

    fn parse_runtime_logs(&self, response: Value) -> Result<Vec<RuntimeLog>> {
        response
            .as_object()
            .and_then(|obj| obj.get("data"))
            .and_then(|data| data.get("runtimeLogs"))
            .and_then(|logs| logs.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
            .iter()
            .map(|log| {
                Ok(RuntimeLog {
                    timestamp: log["timestamp"].as_str().unwrap_or("").to_string(),
                    message: log["message"].as_str().unwrap_or("").to_string(),
                    zeabur_uid: log["zeaburUID"].as_str().unwrap_or("").to_string(),
                })
            })
            .collect()
    }
}
