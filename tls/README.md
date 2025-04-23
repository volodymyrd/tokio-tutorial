# Thread-local storage TLS

Thread-local storage (TLS) is used when each thread needs its own instance of
a variable—independent from other threads.
Let’s break down why you’d need it and when it’s useful.

## 🔁 The Problem: Shared Global State

In a multithreaded program:

* **Global variables** are shared across threads.
* If multiple threads read and write the same global variable without synchronization,
  you get **race conditions** and **data corruption**.

You could fix this with locks…

But sometimes you **don’t want to share** the variable at all—each thread should have
its **own copy**.

## ✅ The Solution: Thread-Local Storage

With TLS, each thread gets its **own separate instance** of a variable.

### Use cases:

1. **Logging contexts**

   Each thread keeps its own log buffer or session context without stepping on each other’s toes.

2. **Database connections / file handles**

   Threads each maintain their own connection or resource state, so they don’t collide.

3. **Random number generators**

   Thread-specific RNGs avoid locking overhead from a shared one.

4. **Caches / Memoization**

   Thread-specific caches improve performance and isolation.

### 💡 Why not just pass data around?

Good question! You could always pass state explicitly. But TLS is helpful when:

* You’re using **deeply nested libraries** that need some shared context.
* You want **ergonomic, global-like access** to something, but per-thread.
* You’re working with **legacy APIs** that expect globals.

## 🔁 TLS in Java and Rust

### 🔷 Java: ThreadLocal<T>

Java has built-in support via the ThreadLocal<T> class.

✅ **Pros**:

* Very **easy to use** — just ThreadLocal<T> and .get()/.set().
* **Garbage collected** — no manual memory management.
* Widely used for things like SimpleDateFormat, request context in servers, etc.

❌ **Cons**:

* Thread-locals can **leak memory** if used in thread pools (e.g. in servlet containers),
  since threads live long and ThreadLocal instances may hang around.
* It's **mutable**, so not as safe — race conditions are possible with bad patterns.
* Less control — Java abstracts away how TLS is implemented.

### 🦀 Rust: thread_local! macro or std::thread::LocalKey

Rust doesn’t have a GC, so TLS is **zero-cost and safe at compile-time**.

✅ **Pros**:

* **Statically checked** — safe access patterns enforced by the compiler.
* **No GC overhead**.
* TLS is **per-thread and truly isolated**, no chance of leaks unless you use unsafe.
* Can be **used in** `no_std` **environments** with #[thread_local].

❌ **Cons**:

* More **manual control** required.
* Syntax can be more verbose.
* If you need to mutate the data, you must use interior mutability (RefCell, Mutex, etc.).
