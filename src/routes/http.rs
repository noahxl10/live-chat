use axum::{routing::get, Router, Json, response::Html, response::Redirect, extract::Request};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/chat") }))
        .route("/chat", get(serve_html))
        .route("/health", get(health_check))
        .route("/test", get(test))
        // .route("/ws", get(ws::web_socket_handler))
}


async fn serve_html(request: Request) -> Html<String> {
    // Try to get the host from the request headers first
    let base_url = request
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_string())
        .or_else(|| std::env::var("BASE_URL").ok())
        .unwrap_or_else(|| "localhost:3000".to_string());

    // Determine protocol - use WSS for HTTPS domains (custom domains and Replit domains)
    // let ws_protocol = if base_url.contains("replit.dev") || base_url.contains("replit.app") || base_url.contains("noahalex.dev") {
    let ws_protocol = if base_url.contains("https") {
        "wss"
    } else {
        "ws"
    };

    let html_content = include_str!("../../static/index.html")
        .replace("{{BASE_URL}}", &base_url)
        .replace("{{WS_PROTOCOL}}", &ws_protocol);

    Html(html_content)
}

async fn health_check() -> Json<&'static str> {
    Json("OK")
}

async fn test() -> Json<&'static str> {
    println!("TEST");
    Json("success")
}