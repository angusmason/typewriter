use tauri::{generate_context, generate_handler, Builder, Manager};
use tauri_plugin_decorum::WebviewWindowExt;

#[cfg_attr(mobile, mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_decorum::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let main_window = app.get_webview_window("main").unwrap();
                main_window.set_traffic_lights_inset(16.0, 24.0).unwrap();
            }
            Ok(())
        })
        .invoke_handler(generate_handler![])
        .run(generate_context!())
        .expect("error while running tauri application");
}
