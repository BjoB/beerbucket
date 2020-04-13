use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IncomingMessage {
    /// request name for function
    /// TODO: Enum Type like in Frontend
    pub request: u8,
    /// content, like parameters
    pub content: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct OutputMessage {
    request: u8,
    message: Option<String>,
    room_list: Option<Vec<String>>,
    error: String,
}

pub fn create_output_message(
    request: u8,
    message: Option<String>,
    room_list: Option<Vec<String>>,
) -> OutputMessage {
    let out_put: OutputMessage = OutputMessage {
        request: request,
        message: message,
        room_list: room_list,
        error: "none".to_string(),
    };
    return out_put;
}
