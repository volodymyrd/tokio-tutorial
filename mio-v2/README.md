### 🚀 There are two separate binaries: one for the server, one for the client

**First**, start the server:

```
cargo run --bin server
```

**Then**, run the client:

```
cargo run --bin client
```

### 😬 The Problem 1

Client version times out because of this critical issue:

> 🔥 Creating the TcpStream in the registry() function,
> but it's dropped immediately when the function ends.

### 🧪 Summary of Key Fix

* ❌ let stream = ... inside a short-lived helper function ⇒ dropped too soon
* ✅ Keep the stream alive for the duration of the event loop

### 😬 The Problem 2

In non-blocking mode, when you call:

```
let mut stream = TcpStream::connect(addr)?;
```

The connect **starts**, but doesn’t finish immediately. It **always returns instantly**,
and you then wait for the socket to become **writable** to know if it connected.

BUT — here's the twist:

> 📌 **A socket becoming writable does not guarantee a successful connection!**

It only means: “the connection attempt has completed — check whether it succeeded or failed.”

🧪 What You Need to Do?
you can use **take_error()**

**🧩 What is a `Token` in mio?**

A `Token` is a simple wrapper around a `usize` — like an ID — used to **uniquely identify registered I/O resources**
(like sockets, listeners, etc.) when events occur.

```
pub struct Token(pub usize);
```

**🧠 Why do we need it?**

When `mio` polls for events, it doesn’t return the actual `TcpStream` or `TcpListener`.
Instead, it returns an `Event`, which includes:

* the **event type** (`READABLE`, `WRITABLE`, etc.)
* the **token** you assigned when registering the socket

So you use the token to **look up** which socket (or resource) the event came from.

**📦 Analogy**

Think of `Poll` as a receptionist and each socket as a visitor. You give each visitor a name tag (token).

Later, when the receptionist says, “Hey, someone is at the door with tag 3,”
you can check your map and go, “Ah, that’s Bob’s socket.”

**🔍 What happens internally?**

When you register a socket:

```
poll.registry().register(&mut socket, Token(42), Interest::READABLE)?;
```

Then during `poll.poll(...)`, if something happens on that socket, you get:

```
Event { token: Token(42), readable: true, writable: false }
```

Now you can match it back to your `clients[Token(42)]`.

**🏁 TL;DR**

* Token is a unique identifier for each I/O resource.
* It's how you match `Events` back to their corresponding sockets.
* Critical in a non-blocking, low-level event-driven model like `mio`.
