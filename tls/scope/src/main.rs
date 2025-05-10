mod scoped;
use crate::scoped::Scoped;

// --- Simple Use Case: Request ID Propagation ---

// This example shows how Scoped<T> allows you to implicitly pass context (the request ID)
// down a call stack (main -> set -> log, main -> set -> process_step -> log)
// without modifying the function signatures of log or process_step.
// The scoping ensures the context is only active when needed and automatically cleaned up.

// Define a thread-local static using the Scoped struct.
// This will hold an optional u64 representing the current request ID.
thread_local! {
    static CURRENT_REQUEST_ID: Scoped<u64> = const {Scoped::new()};
}

/// A simple logging function that includes the current request ID if set.
fn log(message: &str) {
    // Use the thread_local!'s with to get a reference to the Scoped instance,
    // then call the Scoped's with method.
    CURRENT_REQUEST_ID.with(|scoped_instance| {
        scoped_instance.with(|request_id| match request_id {
            Some(id) => println!("[Request ID: {}] {}", id, message),
            None => println!("{}", message),
        });
    });
}

/// A function that simulates some work during request processing.
fn process_step(step_name: &str) {
    log(&format!("Executing step: {}", step_name));
}

fn main() {
    // Log before any request ID is set
    log("Application starting.");

    let request_id_1 = 101;
    // Use the thread_local!'s with to get a reference to the Scoped instance,
    // then call the Scoped's set method.
    CURRENT_REQUEST_ID.with(|scoped_instance| {
        // Set the request ID for the scope of this closure
        scoped_instance.set(&request_id_1, || {
            log("Handling request 101.");
            process_step("Authentication");

            let request_id_2 = 202;
            // Nest another scope with a different request ID
            CURRENT_REQUEST_ID.with(|inner_scoped_instance| {
                inner_scoped_instance.set(&request_id_2, || {
                    log("Handling a nested operation for request 202.");
                    process_step("Sub-process A");
                    process_step("Sub-process B");
                    log("Nested operation finished.");
                }); // The inner Scoped::set scope ends here
            }); // The inner thread_local!::with scope ends here, but doesn't change the Scoped value

            process_step("Authorization");
            log("Request 101 finished.");
        }); // The outer Scoped::set scope ends here, CURRENT_REQUEST_ID is reset to None
    }); // The outer thread_local!::with scope ends here, but doesn't change the Scoped value

    // Log after the request handling scopes have ended
    log("Application shutting down.");
}
