### 🚀 There are two separate binaries: one for the server, one for the client

1. **First**, start the server:

```
cargo run --bin server
```

1. **Then**, run the client:

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
