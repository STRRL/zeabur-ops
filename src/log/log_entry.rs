use std::collections::HashMap;

use chrono::{DateTime, Utc};

// Define the LogEntry struct with public fields
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}
