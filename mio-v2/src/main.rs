use mio::net::TcpStream;
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::net;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    // Create a Poll instance
    let mut poll = Poll::new()?;

    registry(&poll)?;

    // Create a structure to receive polled events
    let mut events = Events::with_capacity(128);

    println!("Starting mio event loop...");

    let start = Instant::now();
    let timeout = Duration::from_millis(500);

    // Main poll loop
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
            if event.token() == Token(0) {
                // Something (probably) happened on the socket.
                println!("Got something on our socket, stopping mio event loop...");
                return Ok(());
            }
        }
    }
}

fn registry(poll: &Poll) -> Result<(), Box<dyn Error>> {
    // Bind a dummy listener on a random port
    let address: net::SocketAddr = "127.0.0.1:0".parse()?;
    let listener = net::TcpListener::bind(address)?;

    // Connect a TcpStream to the listener
    let mut socket = TcpStream::connect(listener.local_addr()?)?;

    // Register the listener with the poller
    poll.registry().register(
        &mut socket,
        Token(0),
        Interest::READABLE | Interest::WRITABLE,
    )?;

    Ok(())
}
