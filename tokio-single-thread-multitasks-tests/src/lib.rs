#[cfg(test)]
mod tests {

    use futures::future::join_all;
    use std::time::Instant;
    use tokio::task::JoinHandle;
    use tokio::time::{Duration, sleep};

    // Use the tokio::test attribute with the "current_thread" flavor.
    // This sets up a single-threaded runtime specifically for this test function.
    #[tokio::test(flavor = "current_thread")]
    async fn test_single_thread_concurrency_with_timing_assertion()
    -> Result<(), Box<dyn std::error::Error>> {
        println!("\nRunning single-threaded Tokio concurrency timing test...");
        let runtime_start_time = Instant::now();

        // Define task durations
        // Make durations large enough to clearly show concurrency benefit
        let duration1_ms = 300;
        let duration2_ms = 200;
        let duration3_ms = 250;

        let simulate_request = |id: u64, duration_ms: u64| async move {
            println!("Task {} started (duration: {}ms)", id, duration_ms);
            sleep(Duration::from_millis(duration_ms)).await;
            println!("Task {} finished", id);
            id // Return the task ID
        };

        // Spawn multiple asynchronous tasks
        let task1: JoinHandle<u64> = tokio::spawn(simulate_request(1, duration1_ms));
        let task2: JoinHandle<u64> = tokio::spawn(simulate_request(2, duration2_ms));
        let task3: JoinHandle<u64> = tokio::spawn(simulate_request(3, duration3_ms));

        // Await the completion of all spawned tasks and collect their results
        let results: Vec<Result<u64, tokio::task::JoinError>> =
            join_all(vec![task1, task2, task3]).await;

        // Process and assert the results of individual tasks
        let mut completed_task_ids: Vec<u64> = results
            .into_iter()
            .map(|res| res.expect("A spawned task panicked")) // Assert that no task panicked
            .collect();

        completed_task_ids.sort();
        assert_eq!(
            completed_task_ids,
            vec![1, 2, 3],
            "All tasks should complete and return their IDs"
        );

        let runtime_end_time = Instant::now();
        let total_test_duration = runtime_end_time.duration_since(runtime_start_time);

        // Calculate the sum of individual task durations
        let sum_of_durations_ms: u64 = duration1_ms + duration2_ms + duration3_ms;
        let sum_of_durations = Duration::from_millis(sum_of_durations_ms);

        println!("Total observed test duration: {:?}", total_test_duration);
        println!("Sum of individual task durations: {:?}", sum_of_durations);

        // *** Assertion for concurrent execution timing ***
        // Assert that the total execution time is less than the sum of individual durations.
        // This demonstrates that the tasks ran concurrently, not sequentially.
        assert!(
            total_test_duration < sum_of_durations,
            "Total test duration ({:?}) should be less than the sum of individual task durations ({:?}) for concurrent execution.",
            total_test_duration,
            sum_of_durations
        );
        // Note: For even stronger assertion (closer to max duration),
        // you could assert total_test_duration is close to max(duration1, duration2, duration3),
        // but that's more susceptible to small timing variations. '< sum' is more robust.

        println!("Test finished successfully, concurrency timing assertion passed.");

        Ok(()) // Return Ok(()) to indicate the test passed
    }
}
