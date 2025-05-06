A single-threaded Tokio runtime excels at handling many concurrent I/O-bound tasks efficiently.
It achieves this concurrency not through parallelism (running tasks on multiple CPU cores simultaneously),
but through cooperative multitasking.
When an asynchronous task on a single-threaded runtime encounters an operation that would block (like
waiting for a network response or a file to be read), it yields control back to the Tokio scheduler.
The scheduler can then pick up another task that is ready to make progress.
This allows a single thread to manage the lifecycle of many tasks, switching between them
whenever one is waiting for an external event.

```
cargo run --bin main_with_macro
```

```
cargo run --bin main_with_builder
```

```
cargo test -- --nocapture
```
