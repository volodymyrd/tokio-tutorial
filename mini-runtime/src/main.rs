use crate::mini_runtime::MiniRuntime;
use std::error::Error;

mod mini_runtime;

fn main() -> Result<(), Box<dyn Error>> {
    let address = "127.0.0.1:9000".parse()?;
    let mut runtime = MiniRuntime::new(address)?;
    runtime.run()
}
