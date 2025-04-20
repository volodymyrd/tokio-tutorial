### 🔄 Goal:

* Server: Accepts connections, reads incoming messages, and echoes them back.
* Client: Sends a message to the server and prints the echoed message.

### 🧠 Key Concepts:

1. We need to manage multiple connections in the server.
2. We’ll use a Slab or a HashMap to store client sockets.
3. We need to handle READABLE and WRITABLE events properly.
4. On the client side, we’ll send data once connected, then wait for the echo.

**First**, start the server:

```
cargo run --bin server
```

**Then**, run the telnet (in multiply console windows) and type something:

```
telnet localhost 9000
```

**🌱 mio: Single-threaded, event-driven loop**

```
loop {
    poll.poll(&mut events, Some(Duration::from_secs(10)))?;
    for event in events.iter() {
        match event.token() {
            SERVER => { /* Accept connection */ }
            Token(n) => { /* Handle client n */ }
        }
    }
}
```

**✅ What this does:**

* Uses **non-blocking I/O** and **event polling**
* Handles **all connections inside one thread**, using the `poll()` event loop
* Only processes **one event at a time**, in the same thread

**🚫 What it doesn’t do:**

* It doesn’t spawn threads or async tasks
* Doesn’t automatically scale across CPUs
