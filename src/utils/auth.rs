
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn maybe_auth(
    request: Request,
    next: Next,
) -> Response {
    // For now, just pass through - implement auth logic here later
    next.run(request).await
}
