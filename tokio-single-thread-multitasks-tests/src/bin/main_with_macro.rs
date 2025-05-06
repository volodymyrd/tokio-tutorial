use std::time::Instant;
use tokio::time::{Duration, sleep};

// Use the tokio::main macro with the "current_thread" flavor
#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Starting single-threaded Tokio runtime test (using #[tokio::main])...");
    let runtime_start_time = Instant::now();

    // An async function that simulates a request or I/O operation
    let simulate_request = |id: u64, duration_ms: u64| async move {
        let start_time = Instant::now();
        println!("Task {} started at {:?}", id, start_time);

        // Simulate work or waiting for an external event
        sleep(Duration::from_millis(duration_ms)).await;

        let end_time = Instant::now();
        println!(
            "Task {} finished at {:?} after {}ms",
            id, end_time, duration_ms
        );
    };

    let mut tasks = Vec::new();

    // Spawn multiple asynchronous tasks
    let task1 = tokio::spawn(simulate_request(1, 2000)); // Task 1 takes 2 seconds
    tasks.push(task1);

    let task2 = tokio::spawn(simulate_request(2, 1000)); // Task 2 takes 1 second
    tasks.push(task2);

    let task3 = tokio::spawn(simulate_request(3, 1500)); // Task 3 takes 1.5 seconds
    tasks.push(task3);

    // Wait for all tasks to complete
    for task in tasks {
        task.await.expect("Task failed");
    }

    let runtime_end_time = Instant::now();
    println!("All tasks finished.");
    println!(
        "Total runtime duration: {:?}",
        runtime_end_time.duration_since(runtime_start_time)
    );
}
