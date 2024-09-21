use zeabur_ops::zeabur::{ZeaburClient, RuntimeLog};
use std::env;
use dotenv::dotenv;
use chrono::Utc;

// Helper function to load environment variables
fn load_env_var(name: &str) -> String {
    dotenv().ok(); // This line loads the .env file
    env::var(name).unwrap_or_else(|_| panic!("{} must be set", name))
}

// Helper function to create a ZeaburClient and load necessary IDs
fn setup_client() -> (ZeaburClient, String, String, String) {
    let api_key = load_env_var("ZEABUR_API_KEY");
    let project_id = load_env_var("ZEABUR_PROJECT_ID");
    let service_id = load_env_var("ZEABUR_SERVICE_ID");
    let environment_id = load_env_var("ZEABUR_ENVIRONMENT_ID");

    let client = ZeaburClient::new(api_key);
    (client, project_id, service_id, environment_id)
}

#[tokio::test]
async fn test_query_service_runtime_logs_without_timestamp() {
    let (client, project_id, service_id, environment_id) = setup_client();

    // Query service runtime logs without timestamp
    let logs = client.query_service_runtime_logs(
        &project_id,
        &service_id,
        &environment_id,
        None,
    ).await.expect("Failed to query service runtime logs");

    // Print the number of logs queried
    println!("Number of logs queried from Zeabur: {}", logs.len());

    // Assert that we received some logs
    assert!(!logs.is_empty(), "No logs were returned");

    // Check the structure of the first log entry
    let first_log = &logs[0];
    assert!(!first_log.timestamp.is_empty(), "Timestamp is empty");
    assert!(!first_log.message.is_empty(), "Message is empty");
    assert!(!first_log.zeabur_uid.is_empty(), "ZeaburUID is empty");

    // Print the first log entry for debugging
    println!("First log entry: {:?}", first_log);
}

#[tokio::test]
async fn test_query_service_runtime_logs_with_timestamp() {
    let (client, project_id, service_id, environment_id) = setup_client();

    // Get the current timestamp
    let current_timestamp = Utc::now().to_rfc3339();

    // Query service runtime logs with current timestamp
    let logs_with_timestamp = client.query_service_runtime_logs(
        &project_id,
        &service_id,
        &environment_id,
        Some(&current_timestamp),
    ).await.expect("Failed to query service runtime logs with timestamp");

    // Print the number of logs queried with timestamp
    println!("Number of logs queried from Zeabur (with timestamp): {}", logs_with_timestamp.len());

    // Check the structure of the first log entry (if any)
    if let Some(first_log) = logs_with_timestamp.first() {
        assert!(!first_log.timestamp.is_empty(), "Timestamp is empty");
        assert!(!first_log.message.is_empty(), "Message is empty");
        assert!(!first_log.zeabur_uid.is_empty(), "ZeaburUID is empty");

        // Print the first log entry for debugging
        println!("First log entry (with timestamp): {:?}", first_log);
    } else {
        println!("No logs returned when using timestamp");
    }
}
