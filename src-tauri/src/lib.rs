use std::fs::write;

use rfd::FileDialog;
use tauri::{command, generate_context, generate_handler, Builder, Manager};
use tauri_plugin_decorum::WebviewWindowExt;

#[command]
fn save_file(data: String) {
    let path = FileDialog::new()
        .set_can_create_directories(true)
        .save_file()
        .unwrap();
    write(path, data).unwrap();
}

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
        .invoke_handler(generate_handler![save_file])
        .run(generate_context!())
        .expect("error while running tauri application");
}
