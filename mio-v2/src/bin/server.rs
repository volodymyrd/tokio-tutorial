use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::net;
use std::time::Duration;

const SERVER: Token = Token(0);

fn main() -> Result<(), Box<dyn Error>> {
    // Create a Poll instance
    let mut poll = Poll::new()?;

    // Register the listener with the poller
    let listener = registry(&poll)?;

    // Create a structure to receive polled events
    let mut events = Events::with_capacity(128);

    println!("Starting mio event loop...");
    loop {
        poll.poll(&mut events, Some(Duration::from_secs(10)))?;

        for event in &events {
            if event.token() == SERVER && event.is_readable() {
                let (_, addr) = listener.accept()?;
                println!("✅ Server accepted connection from {}", addr);

                // Exit after first connection for now
                return Ok(());
            }
        }
    }
}

fn registry(poll: &Poll) -> Result<TcpListener, Box<dyn Error>> {
    // Bind to a specific port
    let address: net::SocketAddr = "127.0.0.1:9000".parse()?;
    let mut listener = TcpListener::bind(address)?;

    // Register the listener with the poller
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    println!("🟢 Server listening on {}", address);

    Ok(listener)
}
