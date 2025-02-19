use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use better_endpoint_protection::{ResponseAction, ThreatEvent};

#[derive(Parser)]
#[command(name = "CLI Client", about = "Sends N threat events to the server in real time")]
struct Cli {
    /// Number of threat events to send
    #[arg(short, long, default_value = "1")]
    requests: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let n = cli.requests;

    let url = "ws://localhost:3000";
    let (socket, _) = connect_async(url).await?;
    println!("WebSocket connection established with {}", url);
    let (mut write, mut read) = socket.split();

    for i in 0..n {
        // Create a threat event with a unique type and a severity.
        let event = ThreatEvent {
            event_type: format!("malware_detected_{}", i + 1),
            severity: ((i % 10) + 1) as i32, // severity from 1 to 10
        };
        let json = serde_json::to_string(&event)?;
        write.send(Message::Text(json)).await?;
        println!("Sent threat event {}: {:?}", i + 1, event);

        // Await the server's response.
        if let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let action: ResponseAction = serde_json::from_str(&text)?;
                    println!("Received response for event {}: {:?}", i + 1, action);
                }
                Ok(_) => println!("Received a non-text message for event {}", i + 1),
                Err(e) => println!("Error receiving response for event {}: {}", i + 1, e),
            }
        } else {
            println!("No response received for event {}", i + 1);
        }

        // Short delay to simulate real-time messaging.
        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
