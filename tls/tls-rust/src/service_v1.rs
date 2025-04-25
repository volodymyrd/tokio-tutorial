use crate::request::Request;
use crate::response::{Response, ResponseStatus};
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::sync::Mutex;
use tracing::{Level, event};

static LOGIN_CONTEXT: std::sync::OnceLock<
    Mutex<HashSet<String, BuildHasherDefault<DefaultHasher>>>,
> = std::sync::OnceLock::new();

fn login_context() -> &'static Mutex<HashSet<String, BuildHasherDefault<DefaultHasher>>> {
    LOGIN_CONTEXT.get_or_init(|| Mutex::new(HashSet::with_hasher(BuildHasherDefault::new())))
}

fn credentials_look_up(username: &str) -> Option<&str> {
    match username {
        "user1" => Some("pass1"),
        "user2" => Some("pass2"),
        _ => None,
    }
}

pub struct Service {}

impl Service {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn get(&self, request: &Request) -> Response {
        event!(Level::INFO, "Got request: {}", request);

        let mut ctx = login_context().lock().unwrap();
        if ctx.contains(request.username()) {
            event!(
                Level::INFO,
                "User {} has been logged in already",
                request.username()
            );
            return Response {
                status: ResponseStatus::SuccessAlreadyLoggedIn,
            };
        }
        if let Some(password) = credentials_look_up(request.username()) {
            if password == request.password() {
                ctx.insert(request.username().to_string());
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
