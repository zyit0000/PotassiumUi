use flate2::write::ZlibEncoder;
use flate2::Compression;

fn compress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed_data = encoder.finish()?;
    Ok(compressed_data)
}

#[tauri::command]
#[allow(non_snake_case)]
async fn OpiumwareExecution(code: String, port: String) -> String {
    let ports = ["8392", "8393", "8394", "8395", "8396", "8397"];
    let mut stream = None;
    let mut connected_port: Option<String> = None;

    let ports_to_check: Vec<String> = match port.as_str() {
        "ALL" => ports.iter().map(|s| s.to_string()).collect(),
        _ => vec![port],
    };    
    
    for port in ports_to_check {
        let server_address = format!("127.0.0.1:{}", port);
        match TcpStream::connect(&server_address) {
            Ok(s) => {
                println!("Successfully connected to Opiumware on port: {}", port);
                stream = Some(s);
                connected_port = Some(port);
                break;
            }
            Err(e) => println!("Failed to connect to port {}: {}", port, e),
        }
    }

    let mut stream = match stream {
        Some(s) => s,
        None => return "Failed to connect on all ports".to_string(),
    };

    fn send_bytes(stream: &mut TcpStream, message: &str) -> Result<(), String> {
        let plaintext = message.as_bytes();
        let compressed = compress_data(plaintext).map_err(|e| e.to_string())?;
        stream.write_all(&compressed).map_err(|e| e.to_string())?;
        println!("Script sent ({} bytes)", compressed.len());
        Ok(())
    }

    if code != "NULL" {
        let message = format!("{}", code);
        if let Err(e) = send_bytes(&mut stream, &message) {
            drop(stream);
            return format!("Error sending script: {}", e);
        }
    }

    drop(stream);
    match connected_port {
        Some(port) => format!("Successfully connected to Opiumware on port: {}", port),
        None => "Failed to connect on all ports".to_string(),
    }
}