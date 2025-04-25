use crate::request::Request;
use crate::response::{Response, ResponseStatus};
use std::cell::RefCell;
use tracing::{Level, event};

fn credentials_look_up(username: &str) -> Option<&str> {
    match username {
        "user1" => Some("pass1"),
        "user2" => Some("pass2"),
        _ => None,
    }
}
thread_local! {
    static LOGIN_CONTEXT: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub struct Service {}

impl Service {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn get(&self, request: &Request) -> Response {
        event!(Level::INFO, "Got request: {}", request);

        if let Some(username) = LOGIN_CONTEXT.take() {
            event!(Level::INFO, "User {} has been logged in already", username);
            return Response {
                status: ResponseStatus::SuccessAlreadyLoggedIn,
            };
        }
        if let Some(password) = credentials_look_up(request.username()) {
            if password == request.password() {
                LOGIN_CONTEXT.set(Some(request.username().to_string()));
                return Response {
                    status: ResponseStatus::Success,
                };
            }
        }
        Response {
            status: ResponseStatus::AuthError,
        }
    }
}
