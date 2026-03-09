// Hello Distributed — 1 coordinator, 20 workers via k3s headless service DNS
use std::env;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

// --- Protocol (shared between coordinator and worker) ---
// Coordinator sends: "TASK:<id>\n"
// Worker responds:   "RESULT:<id>:done\n"

#[tokio::main]
async fn main() {
    let role = env::var("ROLE").unwrap_or_else(|_| "worker".to_string());

    match role.as_str() {
        "coordinator" => run_coordinator().await,
        "worker" => run_worker().await,
        _ => eprintln!("[ERROR] Unknown ROLE: {}", role),
    }
}

// --- COORDINATOR -------------------------------------------------------------

async fn run_coordinator() {
    let workers_env = env::var("WORKERS").unwrap_or_else(|_| {
        "worker-0.worker-headless.default.svc.cluster.local:8080".to_string()
    });

    // Strip any whitespace/newlines injected by the YAML block scalar
    let workers: Vec<String> = workers_env
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    println!("[COORDINATOR] Starting with {} workers", workers.len());

    // Give workers time to start listening
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let mut handles = Vec::with_capacity(workers.len());

    for (i, worker_addr) in workers.iter().enumerate() {
        let addr = worker_addr.clone();
        let task_id = i + 1;

        let handle = tokio::spawn(async move {
            send_task(&addr, task_id).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!(
        "[COORDINATOR] All {} workers responded. Hello Distributed complete.",
        workers.len()
    );
}

async fn send_task(addr: &str, task_id: usize) {
    println!("[COORDINATOR] Connecting to worker at {}", addr);

    let mut stream = loop {
        match TcpStream::connect(addr).await {
            Ok(s) => break s,
            Err(e) => {
                eprintln!(
                    "[COORDINATOR] Could not connect to {}: {}. Retrying in 2s...",
                    addr, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    };

    let task = format!("TASK:{}\n", task_id);
    stream.write_all(task.as_bytes()).await.unwrap();
    println!("[COORDINATOR] Sent to {}: {}", addr, task.trim_end());

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf[..n]);
    println!("[COORDINATOR] Received from {}: {}", addr, response.trim());
}

// --- WORKER ------------------------------------------------------------------

async fn run_worker() {
    let listen_addr =
        env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let listener = TcpListener::bind(&listen_addr).await.unwrap();
    println!("[WORKER] Listening on {}", listen_addr);

    // Loop to accept multiple connections (e.g., coordinator retries or future reuse)
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("[WORKER] Connection from {}", addr);
                tokio::spawn(async move {
                    handle_connection(socket).await;
                });
            }
            Err(e) => {
                eprintln!("[WORKER] Accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buf = vec![0u8; 1024];
    let n = socket.read(&mut buf).await.unwrap();
    let message = String::from_utf8_lossy(&buf[..n]);
    println!("[WORKER] Received: {}", message.trim());

    let response = if message.starts_with("TASK:") {
        // NOTE: trim() on Cow<str> — stable on all editions
        let task_id = message.trim().trim_start_matches("TASK:");
        format!("RESULT:{}:done\n", task_id)
    } else {
        "ERROR:unknown_message\n".to_string()
    };

    socket.write_all(response.as_bytes()).await.unwrap();
    println!("[WORKER] Sent: {}", response.trim());
}

