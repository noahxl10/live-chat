use axum::{
    extract::{
        ws::WebSocketUpgrade,
        ConnectInfo,
        Extension,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use headers::UserAgent;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

mod http;
mod ws;

use crate::state::AppState;

/// Small wrapper that logs, then delegates to the real handler.
async fn ws_with_log(
    ws: WebSocketUpgrade,
    Extension(state): Extension<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
) -> impl IntoResponse {
    println!("*** testing: reached /ws endpoint ***");

    // Call the real handler, forwarding all the same values.
    ws::web_socket_handler(
        ws,
        Extension(state),
        ConnectInfo(addr),
        TypedHeader(user_agent),
    )
    .await
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .merge(http::routes())
        .route("/ws", get(ws_with_log))      // use the wrapper here
        .layer(Extension(state))             // share AppState with all routes
        .layer(TraceLayer::new_for_http())
}