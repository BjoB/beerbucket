use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    ListRooms,
    SetName(String),
    JoinRoom(String),
    ChatMsg { name: String, msg: String },
}

#[derive(Serialize, Deserialize)]
pub struct CommandRequest {
    pub payload: Command,
}

impl CommandRequest {
    /// Return new 'CommandRequest' object from string or error in case of failed parsing.
    pub fn from_str(text: &String) -> Result<Self> {
        serde_json::from_str(text)
    }
}

#[derive(Serialize, Deserialize)]
pub enum ResponsePayload {
    AvailableRooms(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct CommandResponse {
    payload: ResponsePayload,
    error: Option<String>,
}

impl CommandResponse {
    /// Create new 'Response' object from payload and sets error.
    ///
    /// Pass None if no error should be returned.
    pub fn new(payload: ResponsePayload, error: Option<String>) -> Self {
        CommandResponse {
            payload: payload,
            error: error,
        }
    }

    /// Stringify the response.
    pub fn stringify(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_command_request() {
        let example_json_str = String::from(
            r#"
        {
            "payload": "ListRooms"
        }"#,
        );

        let cmd_request = CommandRequest::from_str(&example_json_str).unwrap();
        assert_eq!(cmd_request.payload, Command::ListRooms);
    }

    #[test]
    fn test_basic_response() {
        let example_response = CommandResponse::new(
            ResponsePayload::AvailableRooms(vec![
                String::from("coolroom"),
                String::from("fancyroom"),
            ]),
            None,
        );

        let stringified_response = example_response.stringify();

        println!("{}", stringified_response);
        assert_eq!(stringified_response.len(), 68);
    }
}
