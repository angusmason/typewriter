use std::{
    fs::{read_to_string, write},
    path::{self, PathBuf},
    sync::Mutex,
};

use rfd::FileDialog;
use tauri::{command, generate_context, generate_handler, AppHandle, Builder, Manager, State};
use tauri_plugin_decorum::WebviewWindowExt;

#[command]
fn save_file(data: String, path: Option<PathBuf>) -> Option<PathBuf> {
    let path = match path {
        Some(path) => path,
        None => FileDialog::new()
            .set_can_create_directories(true)
            .save_file()?,
    };
    write(&path, data).unwrap();
    Some(path)
}

#[command]
fn load_file(path: Option<PathBuf>) -> (Option<String>, Option<PathBuf>) {
    let path = match path {
        Some(path) => Some(path),
        None => FileDialog::new().pick_file(),
    };
    match path {
        Some(path) => (read_to_string(&path).ok(), Some(path)),
        None => (None, None),
    }
}

#[command]
fn quit(app_handle: AppHandle) {
    app_handle.exit(0);
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
        .invoke_handler(generate_handler![save_file, load_file, quit])
        .run(generate_context!())
        .expect("error while running tauri application");
}
