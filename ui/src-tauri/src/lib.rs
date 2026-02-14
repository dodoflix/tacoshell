//! Tacoshell Tauri Application
//!
//! This is the main entry point for the Tacoshell desktop application.

mod commands;
mod state;

use state::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tacoshell=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Tacoshell");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new().expect("Failed to initialize app state"))
        .invoke_handler(tauri::generate_handler![
            commands::get_servers,
            commands::add_server,
            commands::update_server,
            commands::delete_server,
            commands::get_secrets,
            commands::add_secret,
            commands::delete_secret,
            commands::link_secret_to_server,
            commands::unlink_secret_from_server,
            commands::connect_ssh,
            commands::disconnect_ssh,
            commands::send_ssh_input,
            commands::resize_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
