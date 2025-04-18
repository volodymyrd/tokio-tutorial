use mio::net::TcpStream;
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::net;
use std::time::{Duration, Instant};

const CLIENT: Token = Token(1);

fn main() -> Result<(), Box<dyn Error>> {
    // Create a Poll instance
    let mut poll = Poll::new()?;

    // Register the stream with the poller
    let stream = registry(&poll)?;

    // Create a structure to receive polled events
    let mut events = Events::with_capacity(128);

    println!("Starting mio event loop...");

    let start = Instant::now();
    let timeout = Duration::from_millis(500);
    loop {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            // Connection timed out
            println!("Connection timed out, stopping mio event loop...");
            return Ok(());
        }

        let remaining = timeout - elapsed;

        // Wait for events
        poll.poll(&mut events, Some(remaining))?;

        for event in &events {
            if event.token() == CLIENT && event.is_writable() {
                // Fix 2 check take_error().
                if let Some(e) = stream.take_error()? {
                    println!("❌ Connection failed: {}. Exiting", e);
                } else {
                    println!(
                        "✅ Client successfully connected from {}!",
                        stream.local_addr()?
                    );
                }
                return Ok(());
            }
        }
    }
}

/// Fix 1 - should return TcpStream, keeping the stream alive for the duration of the event loop.
fn registry(poll: &Poll) -> Result<TcpStream, Box<dyn Error>> {
    // Connect to a specific port
    let address: net::SocketAddr = "127.0.0.1:9000".parse()?;
    let mut stream = TcpStream::connect(address)?;

    // Register the stream with the poller
    poll.registry()
        .register(&mut stream, CLIENT, Interest::WRITABLE)?;

    println!("🔵 Client attempting to connect to {}", address);

    Ok(stream)
}
