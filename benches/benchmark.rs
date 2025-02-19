use criterion::{criterion_group, criterion_main, Criterion};
use criterion::async_executor::AsyncExecutor;
use futures_util::{SinkExt, StreamExt};
use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};
use better_endpoint_protection::{ThreatEvent, ResponseAction};

/// A simple WebSocket server used for benchmarking.
async fn run_server(addr: &str) {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Benchmark server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();
            let (mut write, mut read) = ws_stream.split();
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    // Deserialize the incoming threat event.
                    let _event: ThreatEvent = serde_json::from_str(&text).unwrap();
                    // Immediately respond with a dummy action.
                    let response = ResponseAction {
                        action: "isolate".to_string(),
                    };
                    let json = serde_json::to_string(&response).unwrap();
                    write.send(Message::Text(json)).await.unwrap();
                }
            }
        });
    }
}

/// A WebSocket client that sends `n` threat events.
async fn run_client(addr: &str, n: usize) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("ws://{}", addr);
    let (socket, _) = connect_async(&url).await?;
    let (mut write, mut read) = socket.split();

    for i in 0..n {
        let event = ThreatEvent {
            event_type: format!("test_event_{}", i),
            severity: ((i % 10) + 1) as i32,
        };
        let json = serde_json::to_string(&event)?;
        write.send(Message::Text(json)).await?;
        // Await server response for each request.
        let _ = read.next().await;
    }
    Ok(())
}

/// Wraps a reference to a Tokio runtime to implement AsyncExecutor.
struct TokioExecutor<'a> {
    runtime: &'a Runtime,
}

impl<'a> AsyncExecutor for TokioExecutor<'a> {
    fn block_on<T>(&self, future: impl std::future::Future<Output = T>) -> T {
        self.runtime.block_on(future)
    }
}

/// Benchmark the client sending 100 requests.
fn benchmark_client(c: &mut Criterion) {
    // Create a dedicated Tokio runtime.
    let rt = Runtime::new().unwrap();
    let addr = "127.0.0.1:3001";

    // Spawn the server in a background thread.
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(run_server(addr));
    });

    // Allow the server a moment to start.
    thread::sleep(Duration::from_millis(500));

    // Benchmark sending 100 requests using our TokioExecutor wrapper.
    c.bench_function("client send 100 requests", |b| {
        b.to_async(TokioExecutor { runtime: &rt }).iter(|| async {
            run_client(addr, 100).await.unwrap();
        })
    });
}

criterion_group!(benches, benchmark_client);
criterion_main!(benches);
