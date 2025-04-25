use crate::request::Request;
use crate::request_handler::RequestHandler;
use crate::service_v2::Service;
use std::thread;
use std::time::Duration;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::time::UtcTime;

mod request;
mod request_handler;
mod response;
//mod service_v1;
mod service_v2;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_timer(UtcTime::rfc_3339())
        .with_thread_ids(true) // Enable printing thread IDs
        .with_target(true)
        .fmt_fields(format::DefaultFields::default())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    thread::spawn(|| {
        RequestHandler::new(Service::new(), vec![Request::new("user1", "pass1")]).run()
    });
    thread::spawn(|| {
        RequestHandler::new(Service::new(), vec![Request::new("user2", "pass2")]).run()
    });

    thread::sleep(Duration::from_millis(1000));

    let handle = thread::spawn(|| {
        RequestHandler::new(
            Service::new(),
            vec![
                Request::new("user1", "wrong_pass"),
                Request::new("user1", "pass1"),
                Request::new("user1", "pass1"),
            ],
        )
        .run()
    });

    handle.join().unwrap();
}
