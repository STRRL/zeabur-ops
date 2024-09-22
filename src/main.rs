use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tokio::time::{interval, Duration};
use zeabur_ops::log::{
    log_collector::LogCollector, log_sink::LogSink, sink::otlp_log_sink::OtlpLogSink,
    zeabur_log_collector::ZeaburServiceLogCollector,
};
use zeabur_ops::zeabur::client::ZeaburClient;

fn get_env_var(key: &str) -> Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("Environment variable {} not found", key))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    env_logger::init();

    // Initialize the ZeaburClient
    let client = ZeaburClient::new(get_env_var("ZEABUR_API_KEY")?);

    // Initialize the OTLP log sink
    let sink = OtlpLogSink::new_http()?;

    // Create an interval for running the process every 5 seconds
    let mut interval = interval(Duration::from_secs(5));

    println!("Starting log collection and sinking process...");

    loop {
        interval.tick().await;

        match collect_and_sink_logs_for_all_services(&client, &sink).await {
            Ok(total_log_count) => println!("Successfully processed {} logs in total", total_log_count),
            Err(e) => eprintln!("Error processing logs: {}", e),
        }
    }
}

async fn collect_and_sink_logs_for_all_services(client: &ZeaburClient, sink: &OtlpLogSink) -> Result<usize> {
    let projects = client.list_projects().await?;
    let mut total_log_count = 0;

    for project in projects {
        println!("Processing project: {} (ID: {})", project.name, project.id);
        
        let environments = client.get_environments_of_project(&project.id).await?;
        let services = client.get_services_of_project(&project.id, &environments.environments[0].id).await?;

        for service in services {
            for environment in &environments.environments {
                let collector = ZeaburServiceLogCollector::new(
                    project.id.clone(),
                    environment.id.clone(),
                    service.id.clone(),
                    client.clone(),
                );

                match collect_and_sink_logs(&collector, sink).await {
                    Ok(log_count) => {
                        println!(
                            "Processed {} logs for Project: {} (ID: {}), Service: {} (ID: {}), Environment: {} (ID: {})",
                            log_count, project.name, project.id, service.name, service.id, environment.name, environment.id
                        );
                        total_log_count += log_count;
                    },
                    Err(e) => eprintln!(
                        "Error processing logs for Project: {} (ID: {}), Service: {} (ID: {}), Environment: {} (ID: {}): {}",
                        project.name, project.id, service.name, service.id, environment.name, environment.id, e
                    ),
                }
            }
        }
    }

    Ok(total_log_count)
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
