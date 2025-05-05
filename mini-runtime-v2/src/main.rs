#[macro_use]
pub mod macros;
mod runtime;
mod task;
mod util;

use crate::task::JoinHandle;
pub use task::spawn;

fn main() {
    runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            let _: JoinHandle<i32> = spawn(async move { 5 + 3 });
            let _: JoinHandle<i32> = spawn(async move { 5 + 3 });
            let _: JoinHandle<i32> = spawn(async move { 5 + 3 });
            let _: JoinHandle<i32> = spawn(async move { 5 + 3 });
        });
}
