use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: cargo run --bin load_test <url> <concurrency> <requests>");
        return;
    }

    let url = &args[1];
    let concurrency: usize = args[2].parse().unwrap();
    let total_requests: usize = args[3].parse().unwrap();

    println!("Running load test on {} with {} concurrent connections and {} total requests",
             url, concurrency, total_requests);

    let client = Arc::new(Client::new());
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut handles = Vec::new();

    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for i in 0..total_requests {
        let client = client.clone();
        let semaphore = semaphore.clone();
        let url = url.to_string();

        let handle = tokio::spawn(async move {
            let permit = semaphore.acquire().await.unwrap();
            
            let response = client.get(&url).send().await;
            
            drop(permit);
            
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        Ok(())
                    } else {
                        Err(format!("HTTP error: {}", resp.status()))
                    }
                }
                Err(e) => {
                    Err(format!("Request error: {}", e))
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                eprintln!("Error: {}", e);
            }
        }
    }

    let duration = start.elapsed();
    let rps = total_requests as f64 / duration.as_secs_f64();

    println!("\nTest completed in {:?}", duration);
    println!("Success: {}", success_count);
    println!("Error: {}", error_count);
    println!("Requests per second: {:.2}", rps);
}
