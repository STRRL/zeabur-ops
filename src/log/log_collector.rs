use anyhow::Error;
use async_trait::async_trait;
use super::log_entry::LogEntry;

#[async_trait]
pub trait LogCollector {
    async fn collect_logs(&self) -> Result<Vec<LogEntry>, Error>;
}
