use crate::request::Request;
use crate::response::ResponseStatus;
use crate::service_v2::Service;
use tracing::{Level, event};

pub struct RequestHandler {
    service: Service,
    requests: Vec<Request>,
}

impl RequestHandler {
    pub fn new(service: Service, requests: Vec<Request>) -> Self {
        Self { service, requests }
    }

    pub fn run(&self) {
        event!(
            Level::INFO,
            "Starting request handler with {} requests",
            self.requests.len()
        );
        for request in &self.requests {
            event!(Level::INFO, "Sending request: {}", request);
            let response = self.service.get(request);
            match response.status {
                ResponseStatus::Success => event!(Level::INFO, "Got response: Success"),
                ResponseStatus::SuccessAlreadyLoggedIn => {
                    event!(Level::INFO, "Got response: SuccessAlreadyLoggedIn")
                }
                ResponseStatus::AuthError => println!("Got response: AuthError"),
            }
        }
    }
}
