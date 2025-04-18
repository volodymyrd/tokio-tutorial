use mio::net::TcpStream;
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::time::Duration;
use std::{net, thread};

const CLIENT: Token = Token(1);
const RETRY_INTERVAL: Duration = Duration::from_millis(500);

fn main() -> Result<(), Box<dyn Error>> {
    let address: net::SocketAddr = "127.0.0.1:9000".parse()?;

    loop {
        println!("🔁 Attempting to connect to {}", address);

        // Try to open socket
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                // Create a Poll instance
                let mut poll = Poll::new()?;
                // Create a structure to receive polled events
                let mut events = Events::with_capacity(128);
                // Register the stream with the poller
                poll.registry()
                    .register(&mut stream, CLIENT, Interest::WRITABLE)?;

                println!("Starting mio event loop...");

                // Wait until socket becomes writable
                'poll_loop: loop {
                    // Wait for events
                    poll.poll(&mut events, Some(Duration::from_secs(2)))?;

                    for event in &events {
                        if event.token() == CLIENT && event.is_writable() {
                            if let Some(e) = stream.take_error()? {
                                println!("❌ Connection failed: {}. Retrying...", e);
                                break 'poll_loop;
                            } else {
                                println!(
                                    "✅ Client successfully connected from {}!",
                                    stream.local_addr()?
                                );
                                return Ok(());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Connect failed: {}. Retrying...", e);
            }
        }

        // Wait before retrying
        thread::sleep(RETRY_INTERVAL);
    }
}
