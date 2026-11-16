use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

fn main() {
    println!("Connecting to Ollama 127.0.0.1:11434...");
    let addr: SocketAddr = "127.0.0.1:11434".parse().unwrap();
    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(2000)) {
        Ok(s) => s,
        Err(e) => {
            println!("Connect error: {:?}", e);
            return;
        }
    };
    println!("Connected! Setting timeouts...");
    stream.set_read_timeout(Some(Duration::from_millis(25000))).unwrap();
    stream.set_write_timeout(Some(Duration::from_millis(2000))).unwrap();

    let messages = vec![
        serde_json::json!({
            "role": "system",
            "content": "You are Jarvis."
        }),
        serde_json::json!({
            "role": "user",
            "content": "Hello Jarvis"
        })
    ];

    let body = serde_json::json!({
        "model": "gemma3:1b",
        "messages": messages,
        "stream": false,
        "options": {
            "temperature": 0.7,
            "top_p": 0.9,
            "num_ctx": 8192,
            "num_predict": 250
        }
    }).to_string();

    let body_bytes = body.as_bytes();

    let http_req = format!(
        "POST /api/chat HTTP/1.1\r\nHost: 127.0.0.1:11434\r\nUser-Agent: TRM-Host\r\nAccept: */*\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body_bytes.len(),
        body
    );

    println!("Writing request ({} bytes)...", http_req.len());
    if let Err(e) = stream.write_all(http_req.as_bytes()) {
        println!("Write error: {:?}", e);
        return;
    }
    println!("Request written. Reading response...");

    let mut response_buf = Vec::new();
    match stream.read_to_end(&mut response_buf) {
        Ok(n) => println!("Read {} bytes successfully!", n),
        Err(e) => {
            println!("Read error: {:?}", e);
            return;
        }
    }

    let resp_str = String::from_utf8_lossy(&response_buf);
    println!("Header snippet: {:?}", &resp_str[..200.min(resp_str.len())]);
    if let Some(pos) = resp_str.find("\r\n\r\n") {
        let json_part = &resp_str[pos + 4..];
        match serde_json::from_str::<serde_json::Value>(json_part) {
            Ok(val) => {
                let content = val.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str());
                println!("Extracted content: {:?}", content);
            }
            Err(e) => println!("JSON parse error: {:?}", e),
        }
    } else {
        println!("No double CRLF found in response!");
    }
}
