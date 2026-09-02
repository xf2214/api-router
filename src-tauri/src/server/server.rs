use tokio_util::sync::CancellationToken;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

use super::handlers::{chat_completions, completions, embeddings, health_check, list_models};

/// 构建本地服务路由。
///
/// `enable_cors` 来自配置：为 true 时允许任意来源跨域访问
/// （浏览器端 OpenAI 客户端直连本机端点的核心场景）；为 false 时完全不挂 CORS 层，
/// 仅同源/非浏览器客户端可用。
pub fn build_router(state: AppState, enable_cors: bool) -> Router {
    let mut router = Router::new()
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/completions", post(completions))
        .route("/v1/embeddings", post(embeddings))
        .route("/health", get(health_check));

    if enable_cors {
        router = router.layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    }

    router.with_state(state)
}

pub async fn start_server(
    state: AppState,
    port: u16,
    cancel: CancellationToken,
) -> Result<std::net::SocketAddr, crate::error::AppError> {
    // 从配置读取绑定地址与 CORS 开关；bind_address 解析失败时回退到 127.0.0.1。
    let (bind_ip, enable_cors) = {
        let cfg = state.inner.config.read().await;
        let ip = cfg
            .bind_address
            .parse::<std::net::IpAddr>()
            .unwrap_or(std::net::IpAddr::from([127, 0, 0, 1]));
        (ip, cfg.enable_cors)
    };
    let addr = std::net::SocketAddr::new(bind_ip, port);
    let router = build_router(state, enable_cors);

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
