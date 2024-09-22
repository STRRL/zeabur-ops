// Imports
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::client::ZeaburClient;

// Struct definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub once_product: Option<bool>,
    pub latest_deployment: Option<Deployment>,
    pub template: Option<String>,
    pub market_item_code: Option<String>,
    pub marketplace_item: Option<MarketplaceItem>,
    pub spec: Option<ServiceSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deployment {
    pub plan_type: Option<String>,
    pub plan_meta: Option<Value>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceItem {
    pub name: String,
    pub code: String,
    pub icon_url: String,
    pub network_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub icon: Option<String>,
}

// Implementation
impl ZeaburClient {
    pub async fn get_services_of_project(
        &self,
        project_id: &str,
        environment_id: &str,
    ) -> Result<Vec<Service>> {
        // GraphQL query
        let query = r#"
        query GetServicesOfProject($projectID: ObjectID!, $environmentID: ObjectID!) {
          project(_id: $projectID) {
            services {
              _id
              name
              onceProduct
              latestDeployment(environmentID: $environmentID) {
                planType
                planMeta
                status
              }
              template
              marketItemCode
              marketplaceItem {
                name
                code
                iconURL
                networkType
              }
              spec {
                icon
              }
            }
          }
        }
        "#;

        // Variables for the query
        let variables = serde_json::json!({
            "projectID": project_id,
            "environmentID": environment_id,
        });

        // Execute the query
        let response = self.execute_query(query, variables).await?;
        self.parse_services(response)
    }

    // Helper function to parse the response
    fn parse_services(&self, response: Value) -> Result<Vec<Service>> {
        response
            .as_object()
            .and_then(|obj| obj.get("data"))
            .and_then(|data| data.get("project"))
            .and_then(|project| project.get("services"))
            .and_then(|services| services.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?
            .iter()
            .map(|service| {
                Ok(Service {
                    id: service["_id"].as_str().unwrap_or("").to_string(),
                    name: service["name"].as_str().unwrap_or("").to_string(),
                    once_product: service["onceProduct"].as_bool(),
                    latest_deployment: service["latestDeployment"].as_object().map(|deployment| {
                        Deployment {
                            plan_type: deployment["planType"].as_str().map(String::from),
                            plan_meta: Some(deployment["planMeta"].clone()),
                            status: deployment["status"].as_str().map(String::from),
                        }
                    }),
                    template: service["template"].as_str().map(String::from),
                    market_item_code: service["marketItemCode"].as_str().map(String::from),
                    marketplace_item: service["marketplaceItem"].as_object().map(|item| {
                        MarketplaceItem {
                            name: item["name"].as_str().unwrap_or("").to_string(),
                            code: item["code"].as_str().unwrap_or("").to_string(),
                            icon_url: item["iconURL"].as_str().unwrap_or("").to_string(),
                            network_type: item["networkType"].as_str().map(String::from),
                        }
                    }),
                    spec: service["spec"].as_object().map(|spec| ServiceSpec {
                        icon: spec["icon"].as_str().map(String::from),
                    }),
                })
            })
            .collect()
    }
}
