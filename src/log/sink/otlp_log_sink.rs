use crate::log::log_entry::LogEntry;
use crate::log::log_sink::LogSink;
use anyhow::Error;
use async_trait::async_trait;
use opentelemetry::logs::{LogRecord as OtlpLogRecord, Severity};
use opentelemetry::KeyValue;
use opentelemetry_otlp::{HttpExporterBuilder, LogExporter as OtlpLogExporter, WithExportConfig};
use opentelemetry_sdk::export::logs::{LogBatch, LogExporter};
use opentelemetry_sdk::logs::LogRecord;
use opentelemetry_sdk::{InstrumentationLibrary, Resource};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

// Define the OtlpLogSink struct with resource information
pub struct OtlpLogSink {
    exporter: Arc<Mutex<Box<OtlpLogExporter>>>,
}

impl OtlpLogSink {
    // New constructor using HTTP protocol for vector.dev
    pub fn new_http(labels: HashMap<String, String>) -> Result<Self, Error> {
        let mut exporter = HttpExporterBuilder::default()
            .with_timeout(std::time::Duration::from_secs(3))
            .build_log_exporter()?;

        let kvs: Vec<KeyValue> = labels
            .into_iter()
            .map(|(k, v)| KeyValue::new(k, v))
            .collect();
        let resource = Resource::new(kvs);

        exporter.set_resource(&resource);

        Ok(OtlpLogSink {
            exporter: Arc::new(Mutex::new(Box::new(exporter))),
        })
    }
}

#[async_trait]
impl LogSink for OtlpLogSink {
    async fn store_logs(&self, logs: Vec<LogEntry>) -> Result<(), Error> {
        let instrumentation_library =
            InstrumentationLibrary::builder("opentelemetry-instrumentation-zeabur")
                .with_version(env!("CARGO_PKG_VERSION"))
                .with_schema_url("https://opentelemetry.io/schemas/1.25.0")
                .build();

        let log_records: Vec<(LogRecord, InstrumentationLibrary)> = logs
            .iter()
            .map(|entry| {
                let record: LogRecord = entry.into();
                (record, instrumentation_library.clone())
            })
            .collect();

        let log_records_slice: Vec<(&LogRecord, &InstrumentationLibrary)> =
            log_records.iter().map(|(r, i)| (r, i)).collect();

        let log_batch = LogBatch::new(&log_records_slice);

        // Lock the exporter and get a mutable reference
        let mut guard = self.exporter.lock().await;
        let exporter = guard.as_mut();

        let result = exporter.export(log_batch).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Error exporting logs: {}", e)),
        }
    }
}

// Implement From<LogEntry> for LogRecord
impl From<&LogEntry> for LogRecord {
    fn from(entry: &LogEntry) -> Self {
        let now = SystemTime::now();
        let mut log_record = LogRecord::default();
        log_record.set_body(entry.message.clone().into());
        log_record.set_timestamp(entry.timestamp.into());
        log_record.set_observed_timestamp(now.into());
        log_record.set_severity_number(Severity::Info);
        log_record
    }
}
