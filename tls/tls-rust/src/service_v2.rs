use crate::request::Request;
use crate::response::{Response, ResponseStatus};
use std::cell::RefCell;
use tracing::{Level, event};

fn credentials_look_up(username: &str) -> Option<&'static str> {
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
        match credentials_look_up(request.username()) {
            Some(expected_password) if expected_password == request.password() => {
                LOGIN_CONTEXT.with(|ctx| {
                    *ctx.borrow_mut() = Some(request.username().to_string());
                });

                Response {
                    status: ResponseStatus::Success,
                }
            }
            _ => Response {
                status: ResponseStatus::AuthError,
            },
        }
    }
}
