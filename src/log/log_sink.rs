// Import necessary traits and types
use anyhow::Error;
use async_trait::async_trait;

use super::log_entry::LogEntry;

// Define the LogSink trait
#[async_trait]
pub trait LogSink {
    // Method to store logs
    async fn store_logs(&self, logs: Vec<LogEntry>) -> Result<(), Error>;
}
