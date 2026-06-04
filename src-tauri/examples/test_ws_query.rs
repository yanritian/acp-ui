use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt, Stream};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("=== WebSocket Executive Sessions Query Test ===");

    // Connect to WebSocket server
    let addr = "127.0.0.1:1421";
    println!("Connecting to ws://{}", addr);

    let stream = TcpStream::connect(addr).await.expect("Failed to connect");
    let (ws_stream, _) = tokio_tungstenite::client_async(format!("ws://{}", addr), stream)
        .await
        .expect("WebSocket handshake failed");

    println!("Connected!");

    // Split into sender and receiver using futures_util::StreamExt
    let (mut ws_tx, ws_rx) = ws_stream.split();
    let mut ws_rx = ws_rx.fuse();

    // Send list_executive_sessions command
    let request = json!({
        "id": "query-sessions-1",
        "type": "command",
        "command": "list_executive_sessions",
        "token": null,
        "payload": {
            "limit": 50
        }
    });

    println!("Sending request: {}", request);
    ws_tx.send(Message::Text(request.to_string().into())).await.expect("Failed to send");

    // Wait for response
    println!("Waiting for response...");

    while let Some(msg) = ws_rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let text_str = text.to_string();
                println!("Received: {}", text_str);
                let response: serde_json::Value = serde_json::from_str(&text_str).expect("Failed to parse JSON");
                if response.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                    println!("✅ Success!");
                    if let Some(data) = response.get("data") {
                        if let Some(sessions) = data.get("sessions") {
                            println!("Sessions count: {}", sessions.as_array().map(|a| a.len()).unwrap_or(0));
                            for session in sessions.as_array().unwrap_or(&vec![]) {
                                println!("\n--- Session ---");
                                println!("ID: {}", session.get("id").and_then(|v| v.as_str()).unwrap_or(""));
                                println!("Request: {}", session.get("request").and_then(|v| v.as_str()).unwrap_or("").chars().take(50).collect::<String>());
                                println!("Status: {}", session.get("status").and_then(|v| v.as_str()).unwrap_or(""));
                                println!("Created: {}", session.get("created_at").and_then(|v| v.as_str()).unwrap_or(""));
                            }
                        }
                    }
                    break;
                } else {
                    println!("❌ Error: {}", response.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown"));
                }
            }
            Ok(Message::Close(_)) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("=== Test Complete ===");
}