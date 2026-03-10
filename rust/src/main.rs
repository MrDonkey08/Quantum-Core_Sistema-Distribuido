// Hello Distributed — dynamic worker registration, 1 coordinator, 20 workers
use std::{env, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

// --- Protocol ---
// Worker → Coordinator: "REGISTER:<worker_ip:port>\n"  (on port 9000)
// Coordinator → Worker: "ACK\n"
// Coordinator → Worker: "TASK:<id>\n"                  (on port 8080)
// Worker → Coordinator: "RESULT:<id>:done\n"

const WORKER_COUNT: usize = 20;

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
    let registration_addr = env::var("REGISTRATION_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9000".to_string());

    // Shared list of registered worker addresses
    let workers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let listener = TcpListener::bind(&registration_addr).await.unwrap();
    println!(
        "[COORDINATOR] Listening for worker registrations on {}",
        registration_addr
    );

    // Accept registrations until we have all workers
    while workers.lock().await.len() < WORKER_COUNT {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("[COORDINATOR] Registration connection from {}", addr);
                let workers_clone = Arc::clone(&workers);
                tokio::spawn(async move {
                    handle_registration(socket, workers_clone).await;
                });
            }
            Err(e) => eprintln!("[COORDINATOR] Accept error: {}", e),
        }
    }

    let registered = workers.lock().await.clone();
    println!(
        "[COORDINATOR] All {} workers registered. Dispatching tasks...",
        registered.len()
    );

    // Dispatch tasks to each registered worker
    let mut handles = Vec::with_capacity(registered.len());

    for (i, worker_addr) in registered.iter().enumerate() {
        let addr = worker_addr.clone();
        let task_id = i + 1;

        let handle = tokio::spawn(async move {
            send_task_with_retry(&addr, task_id).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("[COORDINATOR] All tasks complete. Hello Distributed done.");
}

async fn handle_registration(
    mut socket: TcpStream,
    workers: Arc<Mutex<Vec<String>>>,
) {
    let mut buf = vec![0u8; 1024];
    let n = socket.read(&mut buf).await.unwrap();
    let message = String::from_utf8_lossy(&buf[..n]);
    let message = message.trim();

    if let Some(addr) = message.strip_prefix("REGISTER:") {
        let addr = addr.trim().to_string();
        println!("[COORDINATOR] Registered worker at {}", addr);

        let mut list = workers.lock().await;
        list.push(addr);

        socket.write_all(b"ACK\n").await.unwrap();
    } else {
        eprintln!("[COORDINATOR] Unknown registration message: {}", message);
        socket.write_all(b"ERROR:unknown_message\n").await.unwrap();
    }
}

async fn send_task_with_retry(addr: &str, task_id: usize) {
    let max_retries = 3;

    for attempt in 1..=max_retries {
        match try_send_task(addr, task_id).await {
            Ok(_) => return,
            Err(e) => {
                eprintln!(
                    "[COORDINATOR] Task {} failed on {} (attempt {}/{}): {}",
                    task_id, addr, attempt, max_retries, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }

    // TODO: retry on another available worker (fault tolerance)
    eprintln!(
        "[COORDINATOR] Task {} permanently failed after {} attempts on {}",
        task_id, max_retries, addr
    );
}

async fn try_send_task(addr: &str, task_id: usize) -> Result<(), String> {
    let mut stream =
        TcpStream::connect(addr).await.map_err(|e| e.to_string())?;

    let task = format!("TASK:{}\n", task_id);
    stream
        .write_all(task.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    println!("[COORDINATOR] Sent to {}: {}", addr, task.trim_end());

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.map_err(|e| e.to_string())?;
    let response = String::from_utf8_lossy(&buf[..n]);
    println!("[COORDINATOR] Received from {}: {}", addr, response.trim());

    Ok(())
}

// --- WORKER ------------------------------------------------------------------

async fn run_worker() {
    let listen_addr =
        env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let coordinator_addr = env::var("COORDINATOR_ADDR")
        .unwrap_or_else(|_| "10.5.5.1:9000".to_string());

    // Bind listener first so the port is ready before registering
    let listener = TcpListener::bind(&listen_addr).await.unwrap();
    println!("[WORKER] Listening on {}", listen_addr);

    // Get our own pod IP from the env (injected by Kubernetes Downward API)
    let pod_ip = env::var("POD_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let my_addr = format!("{}:8080", pod_ip);

    // Register with coordinator, retrying until it's up
    register_with_coordinator(&coordinator_addr, &my_addr).await;

    // Accept task connections
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("[WORKER] Connection from {}", addr);
                tokio::spawn(async move {
                    handle_connection(socket).await;
                });
            }
            Err(e) => eprintln!("[WORKER] Accept error: {}", e),
        }
    }
}

async fn register_with_coordinator(coordinator_addr: &str, my_addr: &str) {
    println!(
        "[WORKER] Registering with coordinator at {} as {}",
        coordinator_addr, my_addr
    );

    loop {
        match TcpStream::connect(coordinator_addr).await {
            Ok(mut stream) => {
                let msg = format!("REGISTER:{}\n", my_addr);
                if stream.write_all(msg.as_bytes()).await.is_err() {
                    eprintln!(
                        "[WORKER] Failed to send registration. Retrying..."
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(2))
                        .await;
                    continue;
                }

                let mut buf = vec![0u8; 64];
                match stream.read(&mut buf).await {
                    Ok(n) => {
                        let response = String::from_utf8_lossy(&buf[..n]);
                        if response.trim() == "ACK" {
                            println!(
                                "[WORKER] Registration acknowledged by coordinator."
                            );
                            return;
                        } else {
                            eprintln!(
                                "[WORKER] Unexpected ACK response: {}",
                                response.trim()
                            );
                        }
                    }
                    Err(e) => eprintln!("[WORKER] Failed to read ACK: {}", e),
                }
            }
            Err(e) => {
                eprintln!(
                    "[WORKER] Could not connect to coordinator at {}: {}. Retrying...",
                    coordinator_addr, e
                );
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buf = vec![0u8; 1024];
    let n = socket.read(&mut buf).await.unwrap();
    let message = String::from_utf8_lossy(&buf[..n]);
    println!("[WORKER] Received: {}", message.trim());

    let response = if message.starts_with("TASK:") {
        let task_id = message.trim().trim_start_matches("TASK:");
        format!("RESULT:{}:done\n", task_id)
    } else {
        "ERROR:unknown_message\n".to_string()
    };

    socket.write_all(response.as_bytes()).await.unwrap();
    println!("[WORKER] Sent: {}", response.trim());
}

