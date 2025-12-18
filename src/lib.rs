
use axum::body::Body;
use axum::extract::Query;
use axum::http::{Method};
use axum::http::{HeaderValue, Response};
use axum::Json;
use axum::debug_handler;
use axum::{routing::get, Router};
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use tower_service::Service;
use worker::Context;
use worker::Env;
use worker::HttpRequest;
use worker::{console_log, event};

#[event(fetch, respond_with_errors)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> worker::Result<Response<Body>> {
    let cors = CorsLayer::new()
        .allow_origin("https://zhongli.dev".parse::<HeaderValue>()?)
        .allow_methods([Method::GET]);
    let router = Router::new();

    let kv = env
        .kv("api-zhongli")?
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

#[derive(Deserialize)]
struct StickerFilter {
    keyword: Option<String>,
}

async fn get_sticker_list(Query(_filter): Query<StickerFilter>) -> Json<Vec<String>> {
    let sticker_file_id_set = get_sticker_set("", "").await.unwrap_or(vec![]);
    let mut vec = Vec::new();

    for file_id in sticker_file_id_set {
        let path = get_file_path(&file_id, "")
            .await
            .unwrap_or_default();
        vec.push(path);
    }

    Json(vec)
}

#[derive(Deserialize)]
struct GetStickerSetResponse {
    result: GetStickerSetResult,
}

#[derive(Deserialize)]
struct GetStickerSetResult {
    stickers: Vec<Sticker>,
}

#[derive(Deserialize)]
struct Sticker {
    file_id: String,
}

async fn get_sticker_set(
    sticker_set_name: &str,
    bot_token: &str,
) -> Result<Vec<String>, worker::Error> {
    let url = format!(
        "https://api.telegram.org/bot{}/getStickerSet?name={}",
        bot_token, sticker_set_name
    );
    let req = worker::Request::new(&url, worker::Method::Get)?;
    let mut resp = worker::Fetch::Request(req).send().await?;

    let body = resp.json::<GetStickerSetResponse>().await?;
    Ok(body.result
        .stickers
        .into_iter()
        .map(|s| s.file_id)
        .collect()
    )
}

#[derive(Deserialize)]
struct GetFileResponse {
    result: GetFileResult,
}

#[derive(Deserialize)]
struct GetFileResult {
    file_path: String,
}

async fn get_file_path(file_id: &str, bot_token: &str) -> Result<String, worker::Error> {
    let url = format!(
        "https://api.telegram.org/bot{}/getFile?file_id={}",
        bot_token, file_id
    );
    let req = worker::Request::new(&url, worker::Method::Get)?;
    let mut resp = worker::Fetch::Request(req).send().await?;

    let body = resp.json::<GetFileResponse>().await?;

    Ok(format!("https://api.telegram.org/file/bot{}/{}", bot_token, body.result.file_path))
}
