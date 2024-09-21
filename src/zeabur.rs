use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Define the ZeaburClient struct
pub struct ZeaburClient {
    api_key: String,
    client: Client,
}

impl ZeaburClient {
    // Constructor for ZeaburClient
    pub fn new(api_key: String) -> Self {
        let client = Client::new();
        ZeaburClient { api_key, client }
    }

    /// Queries the runtime logs for a specific service in a Zeabur project
    ///
    /// # Arguments
    ///
    /// * `project_id` - The ID of the Zeabur project
    /// * `service_id` - The ID of the service within the project
    /// * `environment_id` - The ID of the environment (e.g., production, staging)
    /// * `timestamp_cursor` - Optional timestamp to fetch logs before this point
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `RuntimeLog` structs or an error
    ///
    /// # Errors
    ///
    /// This function will return an error if the API request fails or if the response
    /// cannot be parsed correctly
    pub async fn query_service_runtime_logs(
        &self,
        project_id: &str,
        service_id: &str,
        environment_id: &str,
        timestamp_cursor: Option<&str>,
    ) -> Result<Vec<RuntimeLog>, Box<dyn std::error::Error>> {
        // Construct the GraphQL query with the optional timestampCursor
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

    // Helper method to execute GraphQL queries
    async fn execute_query(&self, query: &str, variables: serde_json::Value) -> Result<Value, Box<dyn std::error::Error>> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });

        let response = self.client
            .post("https://gateway.zeabur.com/graphql")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(response)
    }

    // Helper method to parse runtime logs from the response
    fn parse_runtime_logs(&self, response: Value) -> Result<Vec<RuntimeLog>, Box<dyn std::error::Error>> {
        let logs = response.as_object().unwrap()
            .get("data")
            .unwrap()
            .get("runtimeLogs")
            .unwrap()
            .as_array()
            .ok_or("Invalid response format")?
            .iter()
            .map(|log| RuntimeLog {
                timestamp: log["timestamp"].as_str().unwrap_or("").to_string(),
                message: log["message"].as_str().unwrap_or("").to_string(),
                zeabur_uid: log["zeaburUID"].as_str().unwrap_or("").to_string(),
            })
            .collect();

        Ok(logs)
    }
}

// Updated RuntimeLog struct
#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeLog {
    pub timestamp: String,
    pub message: String,
    #[serde(rename = "zeaburUID")]
    pub zeabur_uid: String,
}

