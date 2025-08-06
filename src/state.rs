
use tokio::sync::broadcast;
use dashmap::DashMap;
use std::time::Instant;
use crate::models::message::ChatMessage;

#[derive(Clone)]
pub struct AppState {
  pub tx: tokio::sync::broadcast::Sender<ChatMessage>,
  pub usernames: DashMap<String, String>,
  pub last_sent: DashMap<String, Instant>,
}

impl AppState {
  pub fn new(buffer: usize) -> Self {
    let (tx, _) = broadcast::channel(buffer);
    Self {
      tx,
      usernames: DashMap::new(),
      last_sent: DashMap::new(),
    }
  }
}
