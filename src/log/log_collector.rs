use anyhow::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::zeabur::client::ZeaburClient;

use super::log_entry::LogEntry;

#[async_trait]
pub trait LogCollector {
    async fn collect_logs(&self) -> Result<Vec<LogEntry>, Error>;
}
