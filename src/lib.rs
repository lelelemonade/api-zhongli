use axum::{routing::get, Router};
use tower_service::Service;
use tower_http::cors::CorsLayer;
use worker::*;

fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin("https://zhongli.dev".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([axum::http::Method::GET]);

    Router::new().route("/", get(root)).layer(cors)
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router().call(req).await?)
}

pub async fn root() -> &'static str {
    "Hello World!"
}
