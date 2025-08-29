use axum::{
  extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Extension, ConnectInfo},
  response::IntoResponse
};
use axum_extra::TypedHeader;
use headers::UserAgent;
use std::{net::SocketAddr, time::{Duration, Instant}};
use futures_util::{SinkExt, StreamExt};

use crate::{models::message::{ChatMessage, ReceivedMessage}, state::AppState, utils::fingerprinter::username_from_fingerprint};


pub async fn web_socket_handler(
  ws: WebSocketUpgrade,
  Extension(state): Extension<AppState>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  TypedHeader(user_agent): TypedHeader<UserAgent>
) -> impl IntoResponse {
  println!("ATTEMPTED A CONNECTION");
  let fingerprint = format!("{:?}{:?}", addr.ip(), user_agent.as_str());

  let username = state.usernames.entry(fingerprint.clone()).or_insert_with(|| {
    username_from_fingerprint(&fingerprint)
  }).clone();

  ws.on_upgrade(move |socket| handle_socket(socket, state, username))
}


async fn handle_socket(stream: WebSocket, state: AppState, username: String) {

  println!("{} connected", username);

  let (mut sender, mut receiver) = stream.split();

  // Load and send chat history to the new client
  match state.database.get_recent_messages(100).await {
    Ok(messages) => {
      for message in messages {
        let json = message.to_json().unwrap();
        if sender.send(Message::Text(json)).await.is_err() {
          return;
        }
      }
    }
    Err(e) => {
      println!("Failed to load chat history: {}", e);
    }
  }

  // send welcome package to specific client
  let username_clone = username.clone();
  let msg = ChatMessage::try_new(&username_clone, "Welcome to the chat!");
  
  let json_message = msg.unwrap().to_json().unwrap();

  let _ = sender.send(Message::Text(json_message)).await;

  // subscribe to broadcast channel
  let mut rx = state.tx.subscribe();

  // send broadcast to this socket
  // this sends all the messages that have been sent so far to the client
  let send_task = tokio::spawn(async move {
    while let Ok(msg) = rx.recv().await {
      let json = serde_json::to_string(&msg).unwrap();
      if sender.send(Message::Text(json)).await.is_err() {
        break;
      }
    }
  });

  // Receive loop: read messages from client & broadcast them to other clients
  let tx = state.tx.clone();
  let time_last_message_sent_by_user = state.last_sent.clone();
  

  // receive messages from the client and broadcast them to other clients
  let database = state.database.clone();
  let receive_task = tokio::spawn(async move {
    // receive task is responsible for receiving messages from the client and broadcasting them to other clients
    
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
      let received_msg = ReceivedMessage::try_new(&text);

      let now = Instant::now();

      // finds the last time the user sent a message, if it exists
      if let Some(last_sent) = time_last_message_sent_by_user.get(&username_clone) {
        if now.duration_since(*last_sent) < Duration::from_millis(250) {
          continue;
        }
      }

      time_last_message_sent_by_user.insert(username_clone.clone(), now);
      if let Ok(received_msg) = received_msg {
        // Move the `to_chat_message` call here, on `received_msg` which is of type `ReceivedMessage`
        let msg = received_msg.to_chat_message(username_clone.clone());
        if let Ok(message) = msg {
          println!("{}", username_clone);
          
          // Save message to database
          if let Err(e) = database.save_message(&message).await {
            println!("Failed to save message to database: {}", e);
          }
          
          let _ = tx.send(message);
        }
      }
    }
  });

  let _ = tokio::join!(send_task, receive_task);

}
