use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use better_endpoint_protection::{ResponseAction, ThreatEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://localhost:3000";
    let (socket, _) = connect_async(url).await?;
    println!("WebSocket connection established with {}", url);
    let (mut write, mut read) = socket.split();

    // Simulate sending a threat event.
    let event = ThreatEvent {
        event_type: "malware_detected".to_string(),
        severity: 5,
    };
    let json = serde_json::to_string(&event)?;
    write.send(Message::Text(json)).await?;

    // Listen for responses from the server.
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            let action: ResponseAction = serde_json::from_str(&text)?;
            println!("Received action: {:?}", action);
        }
    }
    Ok(())
}
