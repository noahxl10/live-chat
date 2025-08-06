
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


pub const MAX_MESSAGE_SIZE: usize = 1*1024;  // limit to one KB

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
  pub id: Uuid,
  pub username: String,
  pub body: String,
  pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedMessage {
  pub username: String,
  pub content: String,
}

#[derive(Debug)]
pub enum MessageError {
  MessageTooLarge,
  SerializationError(bincode::Error),
  EmptyBody,
}

impl ReceivedMessage {
    pub fn try_new(body: &str) -> Result<Self, MessageError> {
        // strip whitespace and reject empty payloads
        let trimmed = body.trim();
        if trimmed.is_empty() {
            return Err(MessageError::EmptyBody);
        }

        // parse json
        let json: serde_json::Value = serde_json::from_str(trimmed)
            .map_err(|_| MessageError::EmptyBody)?;

        // build struct
        Ok(Self {
            username: json["username"].as_str().unwrap_or("").to_owned(),
            content:  json["content"].as_str().unwrap_or("").to_owned(),
        })
    }

  pub fn to_chat_message(&self, username: impl Into<String>) -> Result<ChatMessage, MessageError> {
      ChatMessage::try_new(username.into(), self.content.clone())
  }
}

impl From<bincode::Error> for MessageError {
  fn from(err: bincode::Error) -> Self {
    MessageError::SerializationError(err)
  }
}

impl ChatMessage {
  pub fn try_new(
    username: impl Into<String>,
    body: impl Into<String>,
  ) -> Result<Self, MessageError> {
    let msg = Self {
      id: Uuid::new_v4(),
      username: username.into(),
      body: body.into(),
      sent_at: Utc::now(),
    };
    msg.assert_size()?;
    Ok(msg)
  }

  pub fn assert_size(&self) -> Result<(), MessageError> {
    let bytes = bincode::serialized_size(self)? as usize;
    if bytes > MAX_MESSAGE_SIZE {
      return Err(MessageError::MessageTooLarge);
    }
    Ok(())
  }

  pub fn to_json(&self) -> Result<String, serde_json::Error> {
      serde_json::to_string(self)
  }

}
