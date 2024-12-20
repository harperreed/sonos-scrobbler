use dotenv::dotenv;
use log::{info, debug};
use std::env;

/// Initialize the logging system with environment-based configuration
fn setup_logging() {
    // Initialize env_logger with RUST_LOG environment variable
    env_logger::init();
}

/// Load and return configuration from environment variables
fn load_config() -> String {
    // Example of reading a configuration value
    env::var("APP_NAME").unwrap_or_else(|_| "default_name".to_string())
}

fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Setup logging system
    setup_logging();
    
    // Load configuration
    let app_name = load_config();
    
    // Log some messages at different levels
    debug!("Debug message - Configuration loaded");
    info!("Starting {} application", app_name);
    
    // Your application logic will go here
    info!("Application initialized successfully");
}
