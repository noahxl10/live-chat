
use tokio::sync::broadcast;
use dashmap::DashMap;
use std::time::Instant;
use std::sync::Arc;
use crate::models::message::ChatMessage;
use crate::database::Database;

#[derive(Clone)]
pub struct AppState {
  pub tx: tokio::sync::broadcast::Sender<ChatMessage>,
  pub usernames: DashMap<String, String>,
  pub last_sent: DashMap<String, Instant>,
  pub database: Arc<Database>,
}

impl AppState {
  pub fn new(buffer: usize, database: Database) -> Self {
    let (tx, _) = broadcast::channel(buffer);
    Self {
      tx,
      usernames: DashMap::new(),
      last_sent: DashMap::new(),
      database: Arc::new(database),
    }
  }
}
