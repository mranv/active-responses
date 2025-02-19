use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use better_endpoint_protection::{ThreatEvent, ResponseAction};

/// Simulate one client that sends many requests over a single WebSocket connection.
async fn run_client_instance(
    url: &str,
    requests_per_client: usize,
    barrier: Arc<Barrier>,
) {
    // Wait for all clients to be ready
    barrier.wait().await;

    let (socket, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = socket.split();

    for i in 0..requests_per_client {
        let event = ThreatEvent {
            event_type: format!("test_event_{}", i),
            severity: ((i % 10) + 1) as i32,
        };
        let json = serde_json::to_string(&event).unwrap();
        write.send(Message::Text(json)).await.expect("Send failed");
        // Optionally, you can await a response:
        let _ = read.next().await;
    }

    // Optionally close the connection gracefully.
    write.close().await.expect("Close failed");
}

/// Runs multiple client instances concurrently.
#[tokio::main]
async fn main() {
    let addr = "ws://localhost:3000";
    let concurrent_clients = 1000;      // Increase this number for more load.
    let requests_per_client = 1000;       // Total requests = concurrent_clients * requests_per_client.
    let barrier = Arc::new(tokio::sync::Barrier::new(concurrent_clients));

    let mut handles = Vec::with_capacity(concurrent_clients);

    for _ in 0..concurrent_clients {
        let barrier_cloned = barrier.clone();
        let addr_cloned = addr.to_string();
        handles.push(tokio::spawn(async move {
            run_client_instance(&addr_cloned, requests_per_client, barrier_cloned).await;
        }));
    }

    // Wait for all client tasks to complete.
    for h in handles {
        let _ = h.await;
    }

    println!("Load test complete.");
}
