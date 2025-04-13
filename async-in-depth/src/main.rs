mod own_future;

use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use crate::own_future::Delay;

#[tokio::main]
async fn main() {
    //how_async_works().await;
    use_my_future().await;
}

async fn how_async_works() {
    let what_is_this = my_async_fn();
    // Nothing has been printed yet.
    println!("-----------");
    what_is_this.await;
    // Text has been printed and socket has been
    // established and closed.
}

async fn my_async_fn() {
    println!("hello from async");
    let _socket = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    println!("async TCP operation complete");
}

async fn use_my_future() {
    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay { when };

    let out = future.await;
    assert_eq!(out, "done");
}
