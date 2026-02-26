use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use std::error::Error;

use flate2::write::ZlibEncoder;
use flate2::Compression;

use tauri::{Manager, Window};

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

fn try_connect(port: &str) -> std::io::Result<TcpStream> {
    let addr = format!("127.0.0.1:{}", port);
    let stream = TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_millis(400),
    )?;
    Ok(stream)
}

fn send_script(stream: &mut TcpStream, code: &str) -> Result<(), Box<dyn Error>> {
    let compressed = compress_data(code.as_bytes())?;
    stream.write_all(&compressed)?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────
// COMMAND: OpiumwareAttach
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
// ─────────────────────────────────────────────────────────────
#[tauri::command]
#[allow(non_snake_case)]
async fn OpiumwareDetach(port: String) -> String {
    println!("[Potassium] Detached from port {}", port);
    format!("Detached from port {}", port)
}

// ─────────────────────────────────────────────────────────────
// COMMAND: check_port
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
    window.set_always_on_top(value).map_err(|e| e.to_string())
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
// Uses rfd (Rusty File Dialogs) — no plugin required.
// ─────────────────────────────────────────────────────────────
#[derive(serde::Serialize)]
struct FileResult {
    name: String,
    content: String,
}

#[tauri::command]
async fn open_file_dialog(_window: Window) -> Result<Option<FileResult>, String> {
    let path = rfd::FileDialog::new()
        .add_filter("Lua Scripts", &["lua", "txt"])
        .add_filter("All Files", &["*"])
        .pick_file();

    match path {
        Some(path_buf) => {
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
// Uses rfd (Rusty File Dialogs) — no plugin required.
// ─────────────────────────────────────────────────────────────
#[tauri::command]
async fn save_file_dialog(
    _window: Window,
    content: String,
    suggested_name: String,
) -> Result<bool, String> {
    let path = rfd::FileDialog::new()
        .set_file_name(&suggested_name)
        .add_filter("Lua Scripts", &["lua", "txt"])
        .save_file();

    match path {
        Some(path_buf) => {
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