mod identity;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            identity::has_identity,
            identity::generate_identity,
            identity::import_identity,
            identity::get_public_key,
            identity::sign,
            identity::list_accounts,
            identity::set_active_account,
            identity::delete_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
