use axum::response::sse::{Event, Sse};
use axum::Json;
use std::convert::Infallible;
use tokio_stream::Stream;
use futures::stream;

pub struct AdjentMcpServer;

impl AdjentMcpServer {
}

pub async fn mcp_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::once(async { Ok(Event::default().data("connected")) });
    Sse::new(stream)
}

pub async fn mcp_post_handler(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(payload)
}
