// Imports
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::ZeaburClient;

// Struct definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub environments: Vec<Environment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

// Implementation
impl ZeaburClient {
    pub async fn get_environments_of_project(&self, project_id: &str) -> Result<Project> {
        // GraphQL query
        let query = r#"
        query GetEnvironmentsOfProject($projectID: ObjectID!) {
          project(_id: $projectID) {
            _id
            name
            environments {
              _id
              name
            }
          }
        }
        "#;

        // Variables for the query
        let variables = serde_json::json!({
            "projectID": project_id,
        });

        // Execute the query
        let response = self.execute_query(query, variables).await?;
        self.parse_project_environments(response)
    }

    // Helper function to parse the response
    fn parse_project_environments(&self, response: Value) -> Result<Project> {
        response
            .as_object()
            .and_then(|obj| obj.get("data"))
            .and_then(|data| data.get("project"))
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))
            .and_then(|project| {
                Ok(Project {
                    id: project["_id"].as_str().unwrap_or("").to_string(),
                    name: project["name"].as_str().unwrap_or("").to_string(),
                    environments: project["environments"]
                        .as_array()
                        .unwrap_or(&Vec::new())
                        .iter()
                        .map(|env| Environment {
                            id: env["_id"].as_str().unwrap_or("").to_string(),
                            name: env["name"].as_str().unwrap_or("").to_string(),
                        })
                        .collect(),
                })
            })
    }
}
