pub struct Response {
    pub(crate) status: ResponseStatus,
}

pub(crate) enum ResponseStatus {
    Success,
    SuccessAlreadyLoggedIn,
    AuthError,
}
