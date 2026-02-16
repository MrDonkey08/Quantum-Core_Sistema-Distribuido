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
    let role = env::var("ROLE").unwrap_or("worker".to_string());

    match role.as_str() {
        "coordinator" => run_coordinator().await,
        "worker" => run_worker().await,
        _ => eprintln!("[ERROR] Unknown ROLE: {}", role),
    }
}

// --- COORDINATOR -------------------------------------------------------------

async fn run_coordinator() {
    let workers_env = env::var("WORKERS")
        .unwrap_or("worker2:8080,worker3:8080,worker4:8080".to_string());

    let workers: Vec<&str> = workers_env.split(',').collect();

    println!("[COORDINATOR] Starting. Workers: {:?}", workers);

    // Give workers time to start listening
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let mut handles = Vec::with_capacity(workers.len());

    for (i, worker_addr) in workers.iter().enumerate() {
        let addr = worker_addr.to_string();
        let task_id = i + 1;

        let handle = tokio::spawn(async move {
            send_task(&addr, task_id).await;
        });

        handles.push(handle);
    }

    // Wait for all workers to respond
    for handle in handles {
        handle.await.unwrap();
    }

    println!(
        "[COORDINATOR] All workers responded. Hello Distributed complete."
    );
}

async fn send_task(addr: &str, task_id: usize) {
    println!("[COORDINATOR] Connecting to worker at {}", addr);

    let mut stream = loop {
        match TcpStream::connect(addr).await {
            Ok(s) => break s,
            Err(e) => {
                eprintln!(
                    "[COORDINATOR] Could not connect to {}: {}. Retrying...",
                    addr, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    };

    let task = format!("TASK:{}\n", task_id);
    stream.write_all(task.as_bytes()).await.unwrap();
    println!("[COORDINATOR] Sent to {}: {}", addr, task.trim_ascii_end());

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf[..n]);
    println!("[COORDINATOR] Received from {}: {}", addr, response.trim());
}

// --- WORKER ------------------------------------------------------------------

async fn run_worker() {
    let listen_addr =
        env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:8080".to_string());

    let listener = TcpListener::bind(&listen_addr).await.unwrap();
    println!("[WORKER] Listening on {}", listen_addr);

    let (socket, addr) = listener.accept().await.unwrap();
    println!("[WORKER] Connection from {}", addr);

    tokio::spawn(async move {
        handle_connection(socket).await;
    });
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buf = vec![0u8; 1024];
    let n = socket.read(&mut buf).await.unwrap();
    let message = String::from_utf8_lossy(&buf[..n]);
    println!("[WORKER] Received: {}", message.trim());

    // Parse "TASK:<id>"
    let response = if message.starts_with("TASK:") {
        let task_id = message.trim_ascii().trim_start_matches("TASK:");
        format!("RESULT:{}:done\n", task_id)
    } else {
        "ERROR:unknown_message\n".to_string()
    };

    socket.write_all(response.as_bytes()).await.unwrap();
    println!("[WORKER] Sent: {}", response.trim());
}
