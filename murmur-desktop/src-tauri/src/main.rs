// Murmur desktop — a Tauri shell over the same web client. The crypto core runs
// as WASM inside the webview, identical to the browser build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running Murmur");
}
