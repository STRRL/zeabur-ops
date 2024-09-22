use super::{log_collector::LogCollector, log_entry::LogEntry};
use crate::zeabur::client::ZeaburClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    async fn collect_logs(&self) -> Result<Vec<LogEntry>, anyhow::Error> {
        self.fetch_logs().await
    }
}

// Constructor and methods for ZeaburServiceLogCollector
impl ZeaburServiceLogCollector {
    // Constructor remains unchanged
    pub fn new(
        project_id: String,
        environment_id: String,
        service_id: String,
        api_key: String,
    ) -> Self {
        let client = ZeaburClient::new(api_key);
        Self {
            project_id,
            environment_id,
            service_id,
            client,
            last_timestamp: Arc::new(Mutex::new(None)),
        }
    }

    // Updated to use LogCollectorError
    async fn fetch_logs(&self) -> Result<Vec<LogEntry>, anyhow::Error> {
        // Retrieve the current last_timestamp
        let mut last_timestamp = self.last_timestamp.lock().await;

        let runtime_logs = self
            .client
            .query_service_runtime_logs(
                &self.project_id,
                &self.service_id,
                &self.environment_id,
                None,
            )
            .await?;

        let mut logs: Vec<LogEntry> = runtime_logs
            .into_iter()
            .filter_map(|log| {
                let timestamp = DateTime::parse_from_rfc3339(&log.timestamp).ok()?;
                let utc_timestamp = timestamp.with_timezone(&Utc);

                // Filter out logs older than or equal to the last_timestamp
                if let Some(last) = *last_timestamp {
                    if utc_timestamp <= last {
                        return None;
                    }
                }

                Some(LogEntry {
                    timestamp: utc_timestamp,
                    message: log.message,
                })
            })
            .collect();

        // Sort logs by timestamp to ensure we get the latest
        logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Update last_timestamp if we have new logs
        if let Some(latest_log) = logs.last() {
            *last_timestamp = Some(latest_log.timestamp);
        }

        Ok(logs)
    }
}
