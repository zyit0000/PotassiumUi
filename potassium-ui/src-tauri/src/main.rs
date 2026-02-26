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
    let compressed_data = encoder.finish()?;
    Ok(compressed_data)
}

// ─────────────────────────────────────────────────────────────
// Port list
// ─────────────────────────────────────────────────────────────
const PORTS: &[&str] = &["8392", "8393", "8394", "8395", "8396", "8397"];

// ─────────────────────────────────────────────────────────────
// COMMAND: OpiumwareAttach
// Scans all ports, returns the first reachable one.
// ─────────────────────────────────────────────────────────────
#[tauri::command]
#[allow(non_snake_case)]
async fn OpiumwareAttach() -> String {
    for port in PORTS {
        let addr = format!("127.0.0.1:{}", port);
        match TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(400)) {
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
// Connects to the given port (or ALL ports) and sends the
// zlib-compressed script. Matches your exact API spec.
// ─────────────────────────────────────────────────────────────
#[tauri::command]
#[allow(non_snake_case)]
async fn OpiumwareExecution(code: String, port: String) -> String {
    let ports_to_check: Vec<String> = match port.as_str() {
        "ALL" => PORTS.iter().map(|s| s.to_string()).collect(),
        _     => vec![port],
    };

    // Inner send helper — mirrors your send_bytes exactly
    fn send_bytes(stream: &mut TcpStream, message: &str) -> Result<(), String> {
        let plaintext  = message.as_bytes();
        let compressed = compress_data(plaintext).map_err(|e| e.to_string())?;
        stream.write_all(&compressed).map_err(|e| e.to_string())?;
        println!("Script sent ({} bytes)", compressed.len());
        Ok(())
    }

    let mut any_success   = false;
    let mut last_error    = String::new();
    let mut success_ports: Vec<String> = Vec::new();

    for p in &ports_to_check {
        let server_address = format!("127.0.0.1:{}", p);
        let addr: std::net::SocketAddr = server_address.parse().unwrap();
        match TcpStream::connect_timeout(&addr, Duration::from_millis(800)) {
            Ok(mut stream) => {
                println!("Successfully connected to Opiumware on port: {}", p);
                if code != "NULL" {
                    match send_bytes(&mut stream, &code) {
                        Ok(_) => {
                            any_success = true;
                            success_ports.push(p.clone());
                        }
                        Err(e) => {
                            last_error = format!("Error sending script: {}", e);
                            eprintln!("[Potassium] {}", last_error);
                        }
                    }
                } else {
                    // NULL = connection test / attach probe
                    any_success = true;
                    success_ports.push(p.clone());
                }
                drop(stream);
            }
            Err(e) => {
                last_error = format!("Failed to connect to port {}: {}", p, e);
                println!("[Potassium] {}", last_error);
            }
        }
    }

    if any_success {
        if success_ports.len() == 1 {
            format!("Successfully connected to Opiumware on port: {}", success_ports[0])
        } else {
            format!("Successfully executed on ports: {}", success_ports.join(", "))
        }
    } else {
        format!("Failed to connect on all ports. Last error: {}", last_error)
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
    let addr = format!("127.0.0.1:{}", port);
    TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(400)).is_ok()
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
            OpiumwareAttach,
            OpiumwareExecution,
            OpiumwareDetach,
            check_port,
            set_always_on_top,
            minimize_window,
            toggle_maximize,
            close_window,
            open_file_dialog,
            save_file_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}