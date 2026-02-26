use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use std::error::Error;

use flate2::write::ZlibEncoder;
use flate2::Compression;

use tauri::{Manager, Window};

#[cfg(target_os = "windows")]
use tauri::window::Effect;

// ─────────────────────────────────────────────────────────────
// Compression helper
// ─────────────────────────────────────────────────────────────
fn compress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

// ─────────────────────────────────────────────────────────────
// Low-level port helpers
// ─────────────────────────────────────────────────────────────
const PORTS: &[&str] = &["8392", "8393", "8394", "8395", "8396", "8397"];

/// Try to open a TCP connection to 127.0.0.1:<port>.
/// Returns Ok(stream) or Err.
fn try_connect(port: &str) -> std::io::Result<TcpStream> {
    let addr = format!("127.0.0.1:{}", port);
    let stream = TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_millis(400),
    )?;
    Ok(stream)
}

/// Send a script over an open stream (zlib-compressed).
fn send_script(stream: &mut TcpStream, code: &str) -> Result<(), Box<dyn Error>> {
    let compressed = compress_data(code.as_bytes())?;
    stream.write_all(&compressed)?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────
// COMMAND: OpiumwareAttach
// Tries all ports, returns result string.
// ─────────────────────────────────────────────────────────────
#[tauri::command]
#[allow(non_snake_case)]
async fn OpiumwareAttach() -> String {
    for port in PORTS {
        match try_connect(port) {
            Ok(_) => {
                println!("[Potassium] Attached on port {}", port);
                return format!("Successfully attached on port {}", port);
            }
            Err(e) => println!("[Potassium] Port {} unavailable: {}", port, e),
        }
    }
    "Failed to attach: no Opiumware instance found on ports 8392-8397".to_string()
}

// ─────────────────────────────────────────────────────────────
// COMMAND: OpiumwareExecution
// Sends a script (or "NULL" as a keep-alive / connection test)
// to one port or all ports.
// ─────────────────────────────────────────────────────────────
#[tauri::command]
#[allow(non_snake_case)]
async fn OpiumwareExecution(code: String, port: String) -> String {
    let ports_to_try: Vec<String> = if port == "ALL" {
        PORTS.iter().map(|s| s.to_string()).collect()
    } else {
        vec![port]
    };

    let mut last_error = String::new();
    let mut any_success = false;
    let mut last_port = String::new();

    for p in &ports_to_try {
        match try_connect(p) {
            Ok(mut stream) => {
                if code != "NULL" {
                    match send_script(&mut stream, &code) {
                        Ok(_) => {
                            println!("[Potassium] Script sent to port {}", p);
                            any_success = true;
                            last_port = p.clone();
                        }
                        Err(e) => {
                            last_error = format!("Error sending to port {}: {}", p, e);
                            eprintln!("[Potassium] {}", last_error);
                        }
                    }
                } else {
                    // NULL = connection test only
                    any_success = true;
                    last_port = p.clone();
                }
            }
            Err(e) => {
                last_error = format!("Cannot connect to port {}: {}", p, e);
                eprintln!("[Potassium] {}", last_error);
            }
        }
    }

    if any_success {
        if ports_to_try.len() == 1 {
            format!("Successfully executed script on port {}", last_port)
        } else {
            "Successfully executed script on all reachable ports".to_string()
        }
    } else {
        format!("Execution failed: {}", last_error)
    }
}

// ─────────────────────────────────────────────────────────────
// COMMAND: OpiumwareDetach
// "Detach" by simply dropping the connection (no data sent).
// The front-end calls this before closing.
// ─────────────────────────────────────────────────────────────
#[tauri::command]
#[allow(non_snake_case)]
async fn OpiumwareDetach(port: String) -> String {
    // Just confirm the port is no longer being held.
    // Since Rust TcpStream drops on close, there's nothing to explicitly undo.
    // If you need to send a disconnect signal to the exploit, do it here.
    println!("[Potassium] Detached from port {}", port);
    format!("Detached from port {}", port)
}

// ─────────────────────────────────────────────────────────────
// COMMAND: check_port
// Returns true if the port is currently open/reachable.
// Used by the front-end dropdown to show green dots.
// ─────────────────────────────────────────────────────────────
#[tauri::command]
async fn check_port(port: String) -> bool {
    try_connect(&port).is_ok()
}

// ─────────────────────────────────────────────────────────────
// COMMAND: set_always_on_top
// ─────────────────────────────────────────────────────────────
#[tauri::command]
async fn set_always_on_top(window: Window, value: bool) -> Result<(), String> {
    window
        .set_always_on_top(value)
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────
// COMMAND: minimize_window
// ─────────────────────────────────────────────────────────────
#[tauri::command]
async fn minimize_window(window: Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────
// COMMAND: toggle_maximize
// ─────────────────────────────────────────────────────────────
#[tauri::command]
async fn toggle_maximize(window: Window) -> Result<(), String> {
    if window.is_maximized().map_err(|e| e.to_string())? {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

// ─────────────────────────────────────────────────────────────
// COMMAND: close_window
// ─────────────────────────────────────────────────────────────
#[tauri::command]
async fn close_window(window: Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────
// COMMAND: open_file_dialog
// Opens a native file picker, returns { name, content }.
// ─────────────────────────────────────────────────────────────
#[derive(serde::Serialize)]
struct FileResult {
    name: String,
    content: String,
}

#[tauri::command]
async fn open_file_dialog(window: Window) -> Result<Option<FileResult>, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = window
        .app_handle()
        .dialog()
        .file()
        .add_filter("Lua Scripts", &["lua", "txt"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    match path {
        Some(p) => {
            let path_buf = p.into_path().map_err(|e| e.to_string())?;
            let name = path_buf
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Script".to_string());
            let content = std::fs::read_to_string(&path_buf).map_err(|e| e.to_string())?;
            Ok(Some(FileResult { name, content }))
        }
        None => Ok(None),
    }
}

// ─────────────────────────────────────────────────────────────
// COMMAND: save_file_dialog
// Opens a native save picker and writes the content.
// ─────────────────────────────────────────────────────────────
#[tauri::command]
async fn save_file_dialog(
    window: Window,
    content: String,
    suggested_name: String,
) -> Result<bool, String> {
    use tauri_plugin_dialog::DialogExt;

    let path = window
        .app_handle()
        .dialog()
        .file()
        .set_file_name(&suggested_name)
        .add_filter("Lua Scripts", &["lua", "txt"])
        .blocking_save_file();

    match path {
        Some(p) => {
            let path_buf = p.into_path().map_err(|e| e.to_string())?;
            std::fs::write(&path_buf, content).map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Ok(false),
    }
}

// ─────────────────────────────────────────────────────────────
// MAIN
// ─────────────────────────────────────────────────────────────
fn main() {
    tauri::Builder::default()
        // If you're using the dialog plugin, register it:
        // .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Core exploit commands
            OpiumwareAttach,
            OpiumwareExecution,
            OpiumwareDetach,
            check_port,
            // Window management
            set_always_on_top,
            minimize_window,
            toggle_maximize,
            close_window,
            // File I/O
            open_file_dialog,
            save_file_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}