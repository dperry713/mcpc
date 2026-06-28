use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Worker 'background-job' starting...");
    loop {
        println!("Worker 'background-job' heartbeat...");
        sleep(Duration::from_secs(5)).await;
    }
}
