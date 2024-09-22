use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::ZeaburClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    #[serde(rename = "iconURL")]
    pub icon_url: String,
    #[serde(rename = "_id")]
    pub id: String,
    pub region: Region,
    pub environments: Vec<Environment>,
    #[serde(rename = "ownerAvatarURL")]
    pub owner_avatar_url: String,
    #[serde(rename = "collaboratorAvatarURLs")]
    pub collaborator_avatar_urls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Region {
    pub provider: String,
    pub name: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl ZeaburClient {
    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let query = r#"
        query GetProjects {
          projects {
            edges {
              node {
                name
                description
                iconURL
                _id
                region {
                  provider
                  name
                  id
                }
                environments {
                  _id
                  name
                }
                owner {
                  avatarURL
                }
                collaborators {
                  avatarURL
                }
              }
            }
          }
        }
        "#;

        let variables = serde_json::json!({});

        let response = self.execute_query(query, variables).await?;
        self.parse_projects(response)
    }

    fn parse_projects(&self, response: Value) -> Result<Vec<Project>> {
        response
            .as_object()
            .and_then(|obj| obj.get("data"))
            .and_then(|data| data.get("projects"))
            .and_then(|projects| projects.get("edges"))
            .and_then(|edges| edges.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
            .iter()
            .filter_map(|edge| edge.get("node"))
            .map(|node| {
                Ok(Project {
                    name: node["name"].as_str().unwrap_or("").to_string(),
                    description: node["description"].as_str().unwrap_or("").to_string(),
                    icon_url: node["iconURL"].as_str().unwrap_or("").to_string(),
                    id: node["_id"].as_str().unwrap_or("").to_string(),
                    region: Region {
                        provider: node["region"]["provider"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        name: node["region"]["name"].as_str().unwrap_or("").to_string(),
                        id: node["region"]["id"].as_str().unwrap_or("").to_string(),
                    },
                    environments: node["environments"]
                        .as_array()
                        .unwrap_or(&Vec::new())
                        .iter()
                        .map(|env| Environment {
                            id: env["_id"].as_str().unwrap_or("").to_string(),
                            name: env["name"].as_str().unwrap_or("").to_string(),
                        })
                        .collect(),
                    owner_avatar_url: node["owner"]["avatarURL"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    collaborator_avatar_urls: node["collaborators"]
                        .as_array()
                        .unwrap_or(&Vec::new())
                        .iter()
                        .filter_map(|collab| collab["avatarURL"].as_str())
                        .map(String::from)
                        .collect(),
                })
            })
            .collect()
    }
}
