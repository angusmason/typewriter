use std::{fs::write, path};

use rfd::FileDialog;
use tauri::{command, generate_context, generate_handler, Builder, Manager};
use tauri_plugin_decorum::WebviewWindowExt;

#[command]
fn save_file(data: String, path: Option<String>) -> Option<String> {
    let path = match path {
        Some(path) => path,
        None => FileDialog::new()
            .set_can_create_directories(true)
            .save_file()?
            .to_str()
            .unwrap()
            .to_string(),
    };
    write(&path, data).unwrap();
    Some(path)
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
