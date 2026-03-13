mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::vault::create_vault,
            commands::vault::load_vault,
            commands::vault::save_vault,
            commands::auth::get_user_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
