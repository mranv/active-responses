use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use better_endpoint_protection::{ResponseAction, ThreatEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bind the TCP listener on port 3000.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server listening on 0.0.0.0:3000");

    while let Ok((stream, addr)) = listener.accept().await {
        // Spawn a new task for each connection.
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("New connection from: {}", addr);

    // Upgrade the TCP connection to a WebSocket connection.
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    // Process incoming messages.
    while let Some(msg) = read.next().await {
        match msg {
            Ok(message) => {
                if let Ok(text) = message.into_text() {
                    match serde_json::from_str::<ThreatEvent>(&text) {
                        Ok(event) => {
                            println!("Received threat event: {:?}", event);
                            // Decide on a response (e.g., isolate the endpoint).
                            let response = ResponseAction {
                                action: "isolate".to_string(),
                            };
                            let json = serde_json::to_string(&response)?;
                            write.send(Message::Text(json)).await?;
                        }
                        Err(e) => eprintln!("Failed to parse threat event: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error reading message: {}", e),
        }
    }
    Ok(())
}
