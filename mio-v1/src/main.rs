use mio::{Events, Poll};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    // Create a Poll instance
    let mut poll = Poll::new()?;

    // Create a structure to receive polled events
    let mut events = Events::with_capacity(128);

    println!("Starting mio event loop...");
    // Wait for events, but none will be received because no
    // `event::Source`s have been registered with this `Poll` instance.
    poll.poll(&mut events, Some(Duration::from_millis(500)))?;
    assert!(events.is_empty());

    println!("Poll completed, no events yet.");

    Ok(())
}
