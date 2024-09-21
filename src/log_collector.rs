use async_trait::async_trait;
use crate::zeabur::ZeaburClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

// Define the LogEntry struct with public fields
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

#[async_trait]
pub trait LogCollector {
    async fn collect_logs(&self) -> Vec<LogEntry>;
}

// Define the ZeaburServiceLogCollector struct
pub struct ZeaburServiceLogCollector {
    project_id: String,
    service_id: String,
    environment_id: String,
    client: ZeaburClient,
    last_timestamp: Arc<Mutex<Option<DateTime<Utc>>>>,
}

// Implement the LogCollector trait for ZeaburServiceLogCollector
#[async_trait]
impl LogCollector for ZeaburServiceLogCollector {
    async fn collect_logs(&self) -> Vec<LogEntry> {
        self.fetch_logs().await.unwrap_or_else(|_| Vec::new())
    }
}

// Constructor and methods for ZeaburServiceLogCollector
impl ZeaburServiceLogCollector {
    pub fn new(project_id: String, environment_id: String, service_id: String, api_key: String) -> Self {
        let client = ZeaburClient::new(api_key);
        Self {
            project_id,
            environment_id,
            service_id,
            client,
            last_timestamp: Arc::new(Mutex::new(None)),
        }
    }

    // Async method to fetch logs using ZeaburClient
    async fn fetch_logs(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let mut all_logs = Vec::new();
        let mut timestamp_cursor = None;

        // Lock the last_timestamp once at the beginning
        let mut last_timestamp = self.last_timestamp.lock().await;

        loop {
            let runtime_logs = self.client.query_service_runtime_logs(
                &self.project_id,
                &self.service_id,
                &self.environment_id,
                timestamp_cursor.as_deref(),
            ).await?;

            if runtime_logs.is_empty() {
                break;
            }

            let mut overlap_detected = false;
            let new_logs: Vec<LogEntry> = runtime_logs
                .into_iter()
                .filter_map(|log| {
                    let timestamp = DateTime::parse_from_rfc3339(&log.timestamp)
                        .ok()?
                        .with_timezone(&Utc);
                    
                    // Check for overlap with the last collected timestamp
                    if let Some(last_ts) = *last_timestamp {
                        if timestamp <= last_ts {
                            overlap_detected = true;
                            return None;
                        }
                    }
                    
                    Some(LogEntry {
                        timestamp,
                        message: log.message,
                    })
                })
                .collect();

            if overlap_detected {
                break;
            }

            if new_logs.is_empty() {
                break;
            }

            // Update the last timestamp
            if let Some(newest_log) = new_logs.first() {
                *last_timestamp = Some(newest_log.timestamp);
            }

            all_logs.extend(new_logs);

            // Update the timestamp cursor for the next iteration
            timestamp_cursor = Some(all_logs.last().unwrap().timestamp.to_rfc3339());
        }

        Ok(all_logs)
    }
}
