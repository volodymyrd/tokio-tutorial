use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

const SERVER: Token = Token(0);

pub(crate) struct MiniRuntime {
    poll: Poll,
    events: Events,
    listener: TcpListener,
    clients: HashMap<Token, TcpStream>,
    next_token: usize,
}

impl MiniRuntime {
    pub fn new(address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let poll = Poll::new()?;
        let mut listener = TcpListener::bind(address)?;

        poll.registry()
            .register(&mut listener, SERVER, Interest::READABLE)?;

        let events = Events::with_capacity(128);

        println!("🟢 Echo server listening on {}", address);

        Ok(Self {
            poll,
            events,
            listener,
            clients: HashMap::new(),
            next_token: SERVER.0 + 1,
        })
    }

    pub(crate) fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!(
            "🟢 Mini Tokio Echo Server running on {:?}",
            self.listener.local_addr()?
        );
        loop {
            self.poll
                .poll(&mut self.events, Some(Duration::from_secs(10)))?;

            // ✅ Workaround for borrow checker
            let tokens: Vec<Token> = self.events.iter().map(|event| event.token()).collect();

            for token in tokens {
                match token {
                    SERVER => self.accept_client()?,
                    token => self.handle_client(token)?,
                }
            }
        }
    }

    fn handle_client(&mut self, token: Token) -> Result<(), Box<dyn Error>> {
        if let Some(socket) = self.clients.get_mut(&token) {
            // Read data from client
            let mut buffer = [0; 1024];
            match socket.read(&mut buffer) {
                Ok(0) => {
                    println!("🔌 Connection closed: {:?}", token);
                    self.clients.remove(&token);
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
                    self.clients.remove(&token);
                }
            }
        }
        Ok(())
    }

    fn accept_client(&mut self) -> Result<(), Box<dyn Error>> {
        // Accept new client
        let (mut socket, addr) = self.listener.accept()?;
        println!("✅ New connection from {}", addr);

        let token = Token(self.next_token);
        self.next_token += 1;
        self.poll.registry().register(
            &mut socket,
            token,
            Interest::READABLE.add(Interest::WRITABLE),
        )?;

        self.clients.insert(token, socket);
        Ok(())
    }
}
