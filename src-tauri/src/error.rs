#[derive(Clone, serde::Serialize)]
pub struct CommandError {
    pub kind: String,
    pub message: String,
}

pub fn command_error(kind: &str, message: impl Into<String>) -> CommandError {
    CommandError {
        kind: kind.to_string(),
        message: message.into(),
    }
}
