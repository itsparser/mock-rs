use std::collections::HashSet;
use std::env;
use std::sync::{Arc, Mutex};

use axum::{Extension, extract::Path, Json, Router, routing::get, routing::post};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use crate::cache::LRUCache;
use crate::form::FormData;

mod cache;
mod form;

type FormCache = Arc<Mutex<LRUCache<FormData>>>;
type FormDataCache = Arc<Mutex<LRUCache<Value>>>;

async fn ping() -> &'static str {
    "pong"
}

async fn welcome() -> &'static str {
    "welcome to Mock server to test the load of the application"
}

async fn get_fields(
    Path(form_id): Path<String>,
    Extension(form_cache): Extension<FormCache>
) -> impl IntoResponse {
    let mut form_cache = form_cache.lock().unwrap();
    match form_cache.get(&form_id) {
        Some(form_data) => {
            if form_data.expires_at.unwrap() < Utc::now() {
                // Form data has expired
                form_cache.remove(&form_id);
                (StatusCode::NOT_FOUND, Json(json!({}))).into_response()
            } else {
                Json(form_data.clone()).into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, Json(json!({}))).into_response(),
    }
}

async fn submit_item(
    Path(form_id): Path<String>,
    Extension(form_cache): Extension<FormCache>,
    Extension(form_data_cache): Extension<FormDataCache>,
    Json(mut payload): Json<Value>,
) -> impl IntoResponse {

    let mut form_cache = form_cache.lock().unwrap();
    match form_cache.get(&form_id) {
        Some(form_data) => {
            if form_data.expires_at.unwrap() < Utc::now() {
                // Form data has expired
                form_cache.remove(&form_id);
                (StatusCode::NOT_FOUND, Json(json!({"message": "Invalid Form".to_string()}))).into_response()
            } else {
                let Value::Object(map) = payload.clone() else { todo!() };
                let json_keys_set: HashSet<&String> = map.keys().collect();
                let required_keys_set: HashSet<&String> = form_data.fields.iter().map(|field| &field.name).collect();
                let mut diff = json_keys_set.difference(&required_keys_set);
                if diff.any(|_| return true) {
                    let v : Vec<&String> = diff.into_iter().map(|item| *item).collect();
                    return (StatusCode::BAD_REQUEST, Json(json!({"message": "Invalid Payload".to_string(), "invalid": v}))).into_response()
                }
                let mut form_data_cache = form_data_cache.lock().unwrap();
                let current_timestamp = Utc::now();
                let data_id = current_timestamp.timestamp_millis().to_string();
                form_data_cache.set(data_id.clone(), payload);
                (StatusCode::OK, Json(json!({"message": "Item Created successfully".to_string(), "id": data_id}))).into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, Json(json!({}))).into_response(),
    }
}

async fn create_form(
    // Path(form_id): Path<String>,,
    Extension(form_cache): Extension<FormCache>,
    Json(mut payload): Json<FormData>,
) -> impl IntoResponse {
    let current_timestamp = Utc::now();
    let form_id = current_timestamp.timestamp_millis().to_string();
    if payload.id.is_none() {
        payload.id = Some(form_id.clone());
    }
    if payload.expires_at.is_none() {
        payload.expires_at = Some(current_timestamp + Duration::hours(1));
    }

    let mut form_cache = form_cache.lock().unwrap();
    form_cache.set(form_id.clone(), payload);
    (StatusCode::OK, Json(json!({"message": "Form Created successfully".to_string(), "id": form_id}))).into_response()
}

#[tokio::main]
async fn main() {
    let val = env::var("CAPACITY").unwrap_or("20".to_string());
    let capacity: usize = val.parse().unwrap_or(20);
    let form_cache: FormCache = Arc::new(Mutex::new(LRUCache::new(capacity)));
    let form_data: FormDataCache = Arc::new(Mutex::new(LRUCache::new(capacity*10)));

    let app = Router::new()
        .route("/", get(welcome))
        .route("/ping", get(ping))
        .route("/forms", post(create_form))
        .route("/forms/:form_id/submit", post(submit_item))
        .route("/forms/:form_id/fields", get(get_fields))
        .layer(Extension(form_cache)).layer(Extension(form_data));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server listening on {:?}", listener);
    axum::serve(listener, app).await.unwrap();
}