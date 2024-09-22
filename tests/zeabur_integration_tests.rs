use chrono::Utc;
use dotenv::dotenv;
use std::env;
use zeabur_ops::zeabur::client::ZeaburClient;
use zeabur_ops::zeabur::get_environments_of_project::Project;

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
    let logs = client
        .query_service_runtime_logs(&project_id, &service_id, &environment_id, None)
        .await
        .expect("Failed to query service runtime logs");

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
    let logs_with_timestamp = client
        .query_service_runtime_logs(
            &project_id,
            &service_id,
            &environment_id,
            Some(&current_timestamp),
        )
        .await
        .expect("Failed to query service runtime logs with timestamp");

    // Print the number of logs queried with timestamp
    println!(
        "Number of logs queried from Zeabur (with timestamp): {}",
        logs_with_timestamp.len()
    );

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

#[tokio::test]
async fn test_list_projects() {
    // Load the API key from the environment
    let api_key = load_env_var("ZEABUR_API_KEY");

    // Create a new ZeaburClient
    let client = ZeaburClient::new(api_key);

    // Call the list_projects method
    let projects = client
        .list_projects()
        .await
        .expect("Failed to list projects");

    // Print the number of projects
    println!("Number of projects: {}", projects.len());

    // Assert that we received at least one project
    assert!(!projects.is_empty(), "No projects were returned");

    // Check the structure of the first project
    let first_project = &projects[0];
    assert!(!first_project.name.is_empty(), "Project name is empty");
    assert!(!first_project.id.is_empty(), "Project ID is empty");

    // Print details of the first project for debugging
    println!("First project details:");
    println!("  Name: {}", first_project.name);
    println!("  ID: {}", first_project.id);
    println!("  Description: {}", first_project.description);
    println!("  Icon URL: {}", first_project.icon_url);
    println!("  Region: {:?}", first_project.region);
    println!("  Environments: {:?}", first_project.environments);
    println!("  Owner Avatar URL: {}", first_project.owner_avatar_url);
    println!(
        "  Collaborator Avatar URLs: {:?}",
        first_project.collaborator_avatar_urls
    );

    // Optional: Check if the project ID from the environment variable exists in the list
    let project_id_from_env = load_env_var("ZEABUR_PROJECT_ID");
    let project_exists = projects.iter().any(|p| p.id == project_id_from_env);
    assert!(
        project_exists,
        "Project ID from environment variable not found in the list of projects"
    );
}

#[tokio::test]
async fn test_get_services_of_project() {
    // Load environment variables
    dotenv::dotenv().ok();
    let api_key = std::env::var("ZEABUR_API_KEY").expect("ZEABUR_API_KEY must be set");
    let project_id = std::env::var("ZEABUR_PROJECT_ID").expect("ZEABUR_PROJECT_ID must be set");
    let environment_id =
        std::env::var("ZEABUR_ENVIRONMENT_ID").expect("ZEABUR_ENVIRONMENT_ID must be set");

    // Create a ZeaburClient instance
    let client = ZeaburClient::new(api_key);

    // Call the get_services_of_project function
    let result = client
        .get_services_of_project(&project_id, &environment_id)
        .await;

    // Assert that the result is Ok
    assert!(result.is_ok());

    // Get the services
    let services = result.unwrap();

    // Assert that we got at least one service
    assert!(!services.is_empty());

    // Check the structure of the first service
    let first_service = &services[0];
    assert!(!first_service.id.is_empty());
    assert!(!first_service.name.is_empty());

    // Optional: Print out some information about the services
    for service in &services {
        println!("Service: {} (ID: {})", service.name, service.id);
        if let Some(deployment) = &service.latest_deployment {
            println!("  Latest deployment status: {:?}", deployment.status);
        }
        if let Some(marketplace_item) = &service.marketplace_item {
            println!(
                "  Marketplace item: {} (Code: {})",
                marketplace_item.name, marketplace_item.code
            );
        }
        println!();
    }
}

#[tokio::test]
async fn test_get_environments_of_project() {
    // Load environment variables
    let (client, project_id, _, _) = setup_client();

    // Call the get_environments_of_project function
    let result = client.get_environments_of_project(&project_id).await;

    // Assert that the result is Ok
    assert!(result.is_ok(), "Failed to get environments of project");

    // Get the project
    let project: Project = result.unwrap();

    // Assert that the project ID matches the one we provided
    assert_eq!(project.id, project_id, "Project ID mismatch");

    // Assert that the project name is not empty
    assert!(!project.name.is_empty(), "Project name is empty");

    // Assert that we got at least one environment
    assert!(
        !project.environments.is_empty(),
        "No environments found for the project"
    );

    // Print out information about the project and its environments
    println!("Project: {} (ID: {})", project.name, project.id);
    println!("Environments:");
    for env in &project.environments {
        println!("  - {} (ID: {})", env.name, env.id);
    }

    // Optional: Check if the environment ID from the environment variable exists in the list
    let environment_id_from_env = load_env_var("ZEABUR_ENVIRONMENT_ID");
    let environment_exists = project
        .environments
        .iter()
        .any(|e| e.id == environment_id_from_env);
    assert!(
        environment_exists,
        "Environment ID from environment variable not found in the list of environments"
    );
}
