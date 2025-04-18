### 🔍 What is Poll in Mio?

Poll is the central object in Mio that lets your program **ask the OS**:

> “Is anything ready to do I/O yet (like read/write)?”

It's a **wrapper around the operating system's I/O event notification mechanism**:

* On Linux: it's epoll
* On macOS/BSD: it's kqueue
* On Windows: it's IOCP

You use Poll to **register interest** in some I/O events (like “tell me when this socket is readable”), and then call
.poll() to **wait** for those events to happen.

### 🔁 What is poll.poll(...) doing?

This is how you say:

> “Wait for I/O events to happen. Here’s a list to fill with events. Wait for at most X time.”

* poll.poll(&mut events, Some(Duration::from_secs(1))) waits **up to 1 second**.
* If no events happen, it **returns empty**, which is exactly what happened in our previous example.

### 📦 What is Events?

Events is a container that gets **filled with I/O events** by poll.poll().

Each event in it means:

* “This socket became readable”
* “That socket became writable”
* etc.

### ❓Why was it empty?

Because:

* We didn't register **any interest in any I/O source** (like a socket, file, etc.).
* So the OS has nothing to watch, and nothing becomes “ready.”

It’s like saying:

> “Tell me if anything interesting happens.” But you never told it what to watch for. So nothing ever does.

### 🔜 What's Next?

To make this real, the next step is to:

* Create a socket (e.g. TCP listener)
* Register it with Poll, saying "Tell me when someone tries to connect"

Then poll.poll(...) will actually return events.
