use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

const SERVER: Token = Token(0);

fn main() -> Result<(), Box<dyn Error>> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

    let address: SocketAddr = "127.0.0.1:9000".parse()?;
    let mut listener = TcpListener::bind(address)?;
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    println!("🟢 Echo server listening on {}", address);

    let mut unique_token = Token(SERVER.0 + 1);
    let mut clients: HashMap<Token, TcpStream> = HashMap::new();

    loop {
        poll.poll(&mut events, Some(Duration::from_secs(10)))?;

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    // Accept new client
                    let (mut socket, addr) = listener.accept()?;
                    println!("✅ New connection from {}", addr);

                    let token = next_token(&mut unique_token);
                    poll.registry().register(
                        &mut socket,
                        token,
                        Interest::READABLE.add(Interest::WRITABLE),
                    )?;
                    clients.insert(token, socket);
                }

                token => {
                    if let Some(socket) = clients.get_mut(&token) {
                        // Read data from client
                        let mut buffer = [0; 1024];
                        match socket.read(&mut buffer) {
                            Ok(0) => {
                                println!("🔌 Connection closed: {:?}", token);
                                clients.remove(&token);
                            }
                            Ok(n) => {
                                let received = &buffer[..n];
                                println!(
                                    "📨 Received from {:?}: {}",
                                    token,
                                    String::from_utf8_lossy(received)
                                );
                                socket.write_all(received)?; // Echo back
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                            Err(e) => {
                                eprintln!("❌ Read error: {}", e);
                                clients.remove(&token);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn next_token(token: &mut Token) -> Token {
    let next = Token(token.0);
    token.0 += 1;
    next
}
