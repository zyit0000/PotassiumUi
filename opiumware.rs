use std::error::Error;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use flate2::write::ZlibEncoder;
use flate2::Compression;

const PORTS: &[&str] = &["8392", "8393", "8394", "8395", "8396", "8397"];

fn compress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

fn send_bytes(stream: &mut TcpStream, message: &str) -> Result<(), String> {
    let compressed = compress_data(message.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&compressed).map_err(|e| e.to_string())?;
    Ok(())
}

fn connect_timeout(port: &str, timeout_ms: u64) -> Result<TcpStream, String> {
    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| e.to_string())?;
    TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).map_err(|e| e.to_string())
}

/// Reference implementation of the Opiumware execution API.
/// - `code == "NULL"` is treated as a connection probe (no bytes sent).
/// - `port == "ALL"` broadcasts to all reachable ports (for UI "All Ports").
pub fn opiumware_execute(code: &str, port: &str) -> String {
    let ports_to_check: Vec<String> = match port {
        "ALL" => PORTS.iter().map(|s| s.to_string()).collect(),
        _ => vec![port.to_string()],
    };

    let mut any_success = false;
    let mut last_error = String::new();
    let mut success_ports: Vec<String> = Vec::new();

    for p in &ports_to_check {
        match connect_timeout(p, 800) {
            Ok(mut stream) => {
                if code != "NULL" {
                    match send_bytes(&mut stream, code) {
                        Ok(_) => {
                            any_success = true;
                            success_ports.push(p.clone());
                        }
                        Err(e) => last_error = format!("Error sending script: {}", e),
                    }
                } else {
                    any_success = true;
                    success_ports.push(p.clone());
                }
                drop(stream);

                // For a single-port call (non-ALL), mirror "first success ends the call".
                if port != "ALL" {
                    break;
                }
            }
            Err(e) => last_error = format!("Failed to connect to port {}: {}", p, e),
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

/// Scan ports and return the first reachable one.
pub fn opiumware_attach_any() -> String {
    for p in PORTS {
        if connect_timeout(p, 400).is_ok() {
            return format!("Successfully attached on port {}", p);
        }
    }
    "Failed to attach: no Opiumware instance found on ports 8392-8397".to_string()
}

pub fn opiumware_detach(port: &str) -> String {
    format!("Detached from port {}", port)
}

pub fn opiumware_check_port(port: &str) -> bool {
    connect_timeout(port, 400).is_ok()
}
