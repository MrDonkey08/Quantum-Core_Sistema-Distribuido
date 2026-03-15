// Distributed Mandelbrot — 1 coordinator, 20 workers
// Protocol:
//   Worker → Coordinator : "REGISTER:<pod_ip:8080>\n"
//   Coordinator → Worker : "ACK\n"
//   Coordinator → Worker : "TASK:<id>:<start_row>:<end_row>:<img_side>\n"
//   Worker → Coordinator : "RESULT:<id>:<byte_count>\n<raw_bytes>"
use std::{env, sync::Arc};

use image::{GrayImage, ImageBuffer, Luma};
use num_complex::Complex;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

const WORKER_COUNT: usize = 20;
const IMG_SIDE: u32 = 4000;
const MAX_ITERATIONS: u16 = 256;
const CX_MIN: f64 = -2.0;
const CX_MAX: f64 = 1.0;
const CY_MIN: f64 = -1.5;
const CY_MAX: f64 = 1.5;

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

// Holds a worker's address and the row range it was assigned
struct WorkerTask {
    addr: String,
    task_id: usize,
    start_row: u32,
    end_row: u32,
}

async fn run_coordinator() {
    let registration_addr = env::var("REGISTRATION_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9000".to_string());

    let workers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let listener = TcpListener::bind(&registration_addr).await.unwrap();
    println!(
        "[COORDINATOR] Listening for registrations on {}",
        registration_addr
    );

    let mut registration_handles = Vec::new();

    // Accept exactly WORKER_COUNT connections, no more
    for _ in 0..WORKER_COUNT {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("[COORDINATOR] Registration connection from {}", addr);
                let workers_clone = Arc::clone(&workers);
                let handle = tokio::spawn(async move {
                    handle_registration(socket, workers_clone).await;
                });
                registration_handles.push(handle);
            }
            Err(e) => eprintln!("[COORDINATOR] Accept error: {}", e),
        }
    }

    // Wait for all registration handlers to finish writing to the list
    for handle in registration_handles {
        handle.await.unwrap();
    }

    let registered = workers.lock().await.clone();
    println!(
        "[COORDINATOR] All {} workers registered. Dispatching tasks...",
        registered.len()
    );

    // Divide rows evenly across workers
    let rows_per_worker = IMG_SIDE / WORKER_COUNT as u32;
    let tasks: Vec<WorkerTask> = registered
        .iter()
        .enumerate()
        .map(|(i, addr)| {
            let start_row = i as u32 * rows_per_worker;
            let end_row = if i == WORKER_COUNT - 1 {
                IMG_SIDE
            } else {
                start_row + rows_per_worker
            };
            WorkerTask {
                addr: addr.clone(),
                task_id: i + 1,
                start_row,
                end_row,
            }
        })
        .collect();

    // Dispatch all tasks concurrently and collect pixel rows
    let results: Arc<Mutex<Vec<(u32, Vec<u8>)>>> =
        Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::with_capacity(tasks.len());

    for task in tasks {
        let results_clone = Arc::clone(&results);
        let handle = tokio::spawn(async move {
            match send_task_with_retry(&task).await {
                Some(pixels) => {
                    results_clone.lock().await.push((task.start_row, pixels));
                }
                None => eprintln!(
                    "[COORDINATOR] Task {} permanently failed, rows {}-{} missing",
                    task.task_id, task.start_row, task.end_row
                ),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Merge results into final image
    assemble_image(results.lock().await.clone()).await;
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
        workers.lock().await.push(addr);
        socket.write_all(b"ACK\n").await.unwrap();
    } else {
        eprintln!("[COORDINATOR] Unknown message: {}", message);
        socket.write_all(b"ERROR:unknown_message\n").await.unwrap();
    }
}

async fn send_task_with_retry(task: &WorkerTask) -> Option<Vec<u8>> {
    let max_retries = 3;

    for attempt in 1..=max_retries {
        match try_send_task(task).await {
            Ok(pixels) => return Some(pixels),
            Err(e) => {
                eprintln!(
                    "[COORDINATOR] Task {} failed on {} (attempt {}/{}): {}",
                    task.task_id, task.addr, attempt, max_retries, e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }

    // TODO: retry on another available worker
    eprintln!(
        "[COORDINATOR] Task {} permanently failed after {} attempts",
        task.task_id, max_retries
    );
    None
}

async fn try_send_task(task: &WorkerTask) -> Result<Vec<u8>, String> {
    let mut stream = TcpStream::connect(&task.addr)
        .await
        .map_err(|e| e.to_string())?;

    // Send task line
    let task_msg = format!(
        "TASK:{}:{}:{}:{}\n",
        task.task_id, task.start_row, task.end_row, IMG_SIDE
    );
    stream
        .write_all(task_msg.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    println!(
        "[COORDINATOR] Sent to {}: {}",
        task.addr,
        task_msg.trim_end()
    );

    // Read "RESULT:<id>:<byte_count>\n"
    let mut header_buf = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        stream
            .read_exact(&mut byte)
            .await
            .map_err(|e| e.to_string())?;
        if byte[0] == b'\n' {
            break;
        }
        header_buf.push(byte[0]);
    }

    let header = String::from_utf8_lossy(&header_buf);
    let parts: Vec<&str> = header.trim().split(':').collect();

    // Expecting RESULT:<id>:<byte_count>
    if parts.len() != 3 || parts[0] != "RESULT" {
        return Err(format!("Unexpected response header: {}", header));
    }

    let byte_count: usize = parts[2]
        .parse()
        .map_err(|e: std::num::ParseIntError| e.to_string())?;
    println!(
        "[COORDINATOR] Receiving {} bytes from {}",
        byte_count, task.addr
    );

    // Read raw pixel bytes
    let mut pixels = vec![0u8; byte_count];
    stream
        .read_exact(&mut pixels)
        .await
        .map_err(|e| e.to_string())?;

    println!("[COORDINATOR] Received result for task {}", task.task_id);
    Ok(pixels)
}

async fn assemble_image(mut results: Vec<(u32, Vec<u8>)>) {
    // Sort by start_row so rows are in correct order
    results.sort_by_key(|(start_row, _)| *start_row);

    let mut img: GrayImage = ImageBuffer::new(IMG_SIDE, IMG_SIDE);

    for (start_row, pixels) in results {
        let row_count = pixels.len() as u32 / IMG_SIDE;
        for row_offset in 0..row_count {
            let y = start_row + row_offset;
            for x in 0..IMG_SIDE {
                let idx = (row_offset * IMG_SIDE + x) as usize;
                img.put_pixel(x, y, Luma([pixels[idx]]));
            }
        }
    }

    img.save("/app/output/fractal.png").unwrap();
    println!("[COORDINATOR] Image saved as /app/output/fractal.png");
}

// --- WORKER ------------------------------------------------------------------

async fn run_worker() {
    let listen_addr =
        env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let coordinator_addr = env::var("COORDINATOR_ADDR")
        .unwrap_or_else(|_| "coordinator-service:9000".to_string());

    let listener = TcpListener::bind(&listen_addr).await.unwrap();
    println!("[WORKER] Listening on {}", listen_addr);

    // Inject pod IP via Kubernetes Downward API (see worker-statefulset.yaml)
    let pod_ip = env::var("POD_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let my_addr = format!("{}:8080", pod_ip);

    register_with_coordinator(&coordinator_addr, &my_addr).await;

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
                            println!("[WORKER] Registration acknowledged.");
                            return;
                        }
                    }
                    Err(e) => eprintln!("[WORKER] Failed to read ACK: {}", e),
                }
            }
            Err(e) => eprintln!(
                "[WORKER] Could not connect to coordinator: {}. Retrying...",
                e
            ),
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn handle_connection(mut socket: TcpStream) {
    // Read task header line
    let mut header_buf = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        if socket.read_exact(&mut byte).await.is_err() {
            eprintln!("[WORKER] Failed to read task header");
            return;
        }
        if byte[0] == b'\n' {
            break;
        }
        header_buf.push(byte[0]);
    }

    let header = String::from_utf8_lossy(&header_buf);
    println!("[WORKER] Received: {}", header.trim());

    // Parse "TASK:<id>:<start_row>:<end_row>:<img_side>"
    let parts: Vec<&str> = header.trim().split(':').collect();
    if parts.len() != 5 || parts[0] != "TASK" {
        eprintln!("[WORKER] Malformed task: {}", header);
        socket.write_all(b"ERROR:malformed_task\n").await.unwrap();
        return;
    }

    let task_id: usize = parts[1].parse().unwrap();
    let start_row: u32 = parts[2].parse().unwrap();
    let end_row: u32 = parts[3].parse().unwrap();
    let img_side: u32 = parts[4].parse().unwrap();

    println!(
        "[WORKER] Computing rows {}-{} for task {}",
        start_row, end_row, task_id
    );

    let pixels = compute_rows(start_row, end_row, img_side);

    // Send "RESULT:<id>:<byte_count>\n" then raw bytes
    let header_response = format!("RESULT:{}:{}\n", task_id, pixels.len());
    socket.write_all(header_response.as_bytes()).await.unwrap();
    socket.write_all(&pixels).await.unwrap();

    println!("[WORKER] Sent {} bytes for task {}", pixels.len(), task_id);
}

fn compute_rows(start_row: u32, end_row: u32, img_side: u32) -> Vec<u8> {
    let scale_x = (CX_MAX - CX_MIN) / img_side as f64;
    let scale_y = (CY_MAX - CY_MIN) / img_side as f64;

    let row_count = (end_row - start_row) as usize;
    let mut pixels = vec![0u8; row_count * img_side as usize];

    for row_offset in 0..(end_row - start_row) {
        let y = start_row + row_offset;
        let cy = CY_MIN + y as f64 * scale_y;

        for x in 0..img_side {
            let cx = CX_MIN + x as f64 * scale_x;
            let c = Complex::new(cx, cy);
            let mut z = Complex::new(0f64, 0f64);
            let mut i = 0u16;

            for t in 0..MAX_ITERATIONS {
                if z.norm() > 2.0 {
                    break;
                }
                z = z * z + c;
                i = t;
            }

            let idx = row_offset as usize * img_side as usize + x as usize;
            pixels[idx] = i as u8;
        }
    }

    pixels
}
