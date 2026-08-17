use crate::{
    model::{PushPayload, PushResult, ReceiverStatus},
    receiver::PORT,
    runtime::AppRuntime,
};
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Request, State},
    http::{
        header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN},
        HeaderValue, Method, StatusCode,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};

pub fn start(runtime: Arc<AppRuntime>) {
    tauri::async_runtime::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(("0.0.0.0", PORT)).await {
            Ok(listener) => listener,
            Err(error) => {
                runtime
                    .set_receiver_status(ReceiverStatus::Unavailable {
                        port: PORT,
                        message: "端口 8899 已被占用，接收服务未启动".to_string(),
                    })
                    .await;
                log::error!("HTTP receiver bind failed: {error}");
                return;
            }
        };

        runtime.set_listening_unless_degraded(PORT).await;

        if let Err(error) = axum::serve(listener, router(runtime.clone())).await {
            runtime
                .set_receiver_status(ReceiverStatus::Unavailable {
                    port: PORT,
                    message: "接收服务不可用".to_string(),
                })
                .await;
            log::error!("HTTP receiver serve failed: {error}");
        }
    });
}

fn router(runtime: Arc<AppRuntime>) -> Router {
    Router::new()
        .route("/", get(health).options(options))
        .route("/health", get(health).options(options))
        .route("/push", post(push).options(options))
        .fallback(fallback)
        .layer(DefaultBodyLimit::max(65_536))
        .layer(middleware::from_fn(cors_headers))
        .with_state(runtime)
}

async fn options() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn fallback(method: Method) -> Response {
    if method == Method::OPTIONS {
        return StatusCode::NO_CONTENT.into_response();
    }
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "ok": false, "message": "not found" })),
    )
        .into_response()
}

async fn cors_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("*"));
    headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));
    response
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        name: "sms-bridge-receiver",
        version: "2.0.0",
        host: hostname::get()
            .ok()
            .and_then(|host| host.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string()),
        port: PORT,
        time: unix_millis(),
    })
}

async fn push(State(runtime): State<Arc<AppRuntime>>, body: Bytes) -> Json<PushResult> {
    let payload = serde_json::from_slice(&body).unwrap_or_else(|_| PushPayload {
        text: Some(String::from_utf8_lossy(&body).into_owned()),
        ..Default::default()
    });
    Json(runtime.accept_push(payload).await)
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    name: &'static str,
    version: &'static str,
    host: String,
    port: u16,
    time: u64,
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
