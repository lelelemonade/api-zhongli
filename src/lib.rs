use axum::{routing::get, Router};
use axum::http::{Response,HeaderValue};
use axum::http::Method;
use axum::extract::Query;
use axum::body::Body;
use axum::Json;
use tower_http::cors::CorsLayer;
use tower_service::Service;
use worker::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct StickerFilter {
    keyword: Option<String>,
}

#[event(fetch, respond_with_errors)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<Response<Body>> {
    let cors = CorsLayer::new()
        .allow_origin("https://zhongli.dev".parse::<HeaderValue>()?)
        .allow_methods([Method::GET]);
    let router = Router::new();

    let kv = env.kv("api-zhongli")?
        .get("last-fully-refresh-telegram-sticker-time")
        .text()
        .await?
        .unwrap();
    let secret = env.secret("telegram-sticker-bot-token")?.to_string();

    console_log!("kv: {}, secret:{}", kv, secret);

    Ok(router
        .route("/", get(root))
        .route("/getStickerList", get(get_sticker_list))
        .layer(cors)
        .call(req)
        .await?)
}

pub async fn root() -> &'static str {
    "Hello World!"
}

async fn get_sticker_list(Query(filter): Query<StickerFilter>) -> Json<Vec<String>> {
    Json(vec![filter.keyword.unwrap(), "bar".to_owned()])
}