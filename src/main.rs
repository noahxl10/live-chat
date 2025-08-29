use axum::{
  middleware,
  Extension,
  Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod database;
mod models;
mod routes;
mod state;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::dotenv().ok();
  
  tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(tracing_subscriber::EnvFilter::new(
      std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "example_chat=debug,tower_http=debug".into()),
    ))
    .init();


  // Set up environment-based URLs
  let is_replit_env = std::env::var("REPLIT_DEPLOYMENT").is_ok();
  let base_url = "https://chat.noahalex.dev";
  let ws_protocol = "wss";

  // Set protocol - use WSS for any Replit environment, WS for local only
  let ws_protocol = "wss"; //if is_replit_env { "wss" } else { "wss" };
  
  std::env::set_var("BASE_URL", &base_url);
  std::env::set_var("WS_PROTOCOL", ws_protocol);

  // Initialize database connection
  let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
  let database = database::Database::new(&database_url).await?;

  // shared state
  let app_state = state::AppState::new(100, database);

  let app: Router = routes::routes(app_state.clone())
    .layer(CorsLayer::new().allow_origin(Any))
    .layer(Extension(app_state))
    .layer(middleware::from_fn(utils::auth::maybe_auth));

  let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
  tracing::info!("listening on {}", addr);

  let listener = tokio::net::TcpListener::bind(addr).await?;

  axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
    .await?;

  Ok(())
}