use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tokio::time::{interval, Duration};
use zeabur_ops::log::{
    log_collector::LogCollector, log_sink::LogSink, sink::otlp_log_sink::OtlpLogSink,
    zeabur_log_collector::ZeaburServiceLogCollector,
};

fn get_env_var(key: &str) -> Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("Environment variable {} not found", key))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    env_logger::init();

    // Initialize the Zeabur log collector
    let collector = ZeaburServiceLogCollector::new(
        get_env_var("ZEABUR_PROJECT_ID")?,
        get_env_var("ZEABUR_ENVIRONMENT_ID")?,
        get_env_var("ZEABUR_SERVICE_ID")?,
        get_env_var("ZEABUR_API_KEY")?,
    );

    // Initialize the OTLP log sink
    let sink = OtlpLogSink::new_http()?;

    // Create an interval for running the process every 5 seconds
    let mut interval = interval(Duration::from_secs(5));

    println!("Starting log collection and sinking process...");

    loop {
        interval.tick().await;

        match collect_and_sink_logs(&collector, &sink).await {
            Ok(log_count) => println!("Successfully processed {} logs", log_count),
            Err(e) => eprintln!("Error processing logs: {}", e),
        }
    }
}

async fn collect_and_sink_logs(
    collector: &impl LogCollector,
    sink: &impl LogSink,
) -> Result<usize> {
    // Collect logs
    let logs = collector.collect_logs().await?;
    let log_count = logs.len();

    // Sink logs
    sink.store_logs(logs).await?;

    Ok(log_count)
}
