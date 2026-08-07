use tokio_util::sync::CancellationToken;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

use super::handlers::{
    chat_completions, completions, embeddings, health_check, list_models,
};

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/completions", post(completions))
        .route("/v1/embeddings", post(embeddings))
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(state)
}

pub async fn start_server(
    state: AppState,
    port: u16,
    cancel: CancellationToken,
) -> Result<std::net::SocketAddr, crate::error::AppError> {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let router = build_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router)
            .with_graceful_shutdown(async move { cancel.cancelled().await })
            .await
        {
            tracing::error!("Local server error: {e}");
        }
    });

    Ok(bound_addr)
}
