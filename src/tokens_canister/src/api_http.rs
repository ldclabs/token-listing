use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use candid::CandidType;
use chrono::prelude::*;
use ic_auth_types::cbor_into_vec;
use ic_http_certification::{HeaderField, HttpRequest};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::json;

use crate::store;

#[derive(CandidType, Deserialize, Serialize, Clone, Default)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub upgrade: Option<bool>,
}

struct HttpError {
    status_code: u16,
    message: String,
}

static JSON: &str = "application/json";
static IC_CERTIFICATE_HEADER: &str = "ic-certificate";
static IC_CERTIFICATE_EXPRESSION_HEADER: &str = "ic-certificateexpression";

#[ic_cdk::query(hidden = true)]
async fn http_request(request: HttpRequest<'static>) -> HttpResponse {
    let req_path = request.get_path();
    let witness = store::state::http_tree_with(|t| {
        t.witness(&store::state::DEFAULT_CERT_ENTRY, request.url())
            .expect("get witness failed")
    });

    let certified_data = ic_cdk::api::data_certificate().expect("no data certificate available");

    let mut headers = vec![
        ("x-content-type-options".to_string(), "nosniff".to_string()),
        (
            IC_CERTIFICATE_EXPRESSION_HEADER.to_string(),
            store::state::DEFAULT_CEL_EXPR.clone(),
        ),
        (
            IC_CERTIFICATE_HEADER.to_string(),
            format!(
                "certificate=:{}:, tree=:{}:, expr_path=:{}:, version=2",
                BASE64.encode(certified_data),
                BASE64.encode(cbor_into_vec(&witness).expect("failed to serialize witness")),
                BASE64.encode(
                    cbor_into_vec(&store::state::DEFAULT_EXPR_PATH.to_expr_path())
                        .expect("failed to serialize expr path")
                )
            ),
        ),
    ];

    let req_path = match req_path {
        Ok(path) => path,
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            return HttpResponse {
                status_code: 400,
                headers,
                body: err.to_string().into_bytes().into(),
                upgrade: None,
            };
        }
    };

    let rt = match (request.method().as_str(), req_path.as_str()) {
        ("HEAD", _) => Ok(Vec::new()),
        ("GET", "/") => get_info(),
        ("GET", "/uniswap/all.json") => get_tokens(),
        ("GET", path) if path.ends_with(".json") => get_token(path),
        (method, path) => Err(HttpError {
            status_code: 404,
            message: format!("method {method}, path: {path}"),
        }),
    };

    match rt {
        Ok(body) => {
            headers.push(("content-type".to_string(), JSON.to_string()));
            headers.push(("content-length".to_string(), body.len().to_string()));
            HttpResponse {
                status_code: 200,
                headers,
                body: body.into(),
                upgrade: None,
            }
        }
        Err(err) => {
            headers.push(("content-type".to_string(), "text/plain".to_string()));
            HttpResponse {
                status_code: err.status_code,
                headers,
                body: err.message.into_bytes().into(),
                upgrade: None,
            }
        }
    }
}

fn get_info() -> Result<Vec<u8>, HttpError> {
    let body = store::state::info();
    serde_json::to_vec(&body).map_err(|err| HttpError {
        status_code: 500,
        message: format!("failed to serialize state, error: {err}"),
    })
}

fn get_tokens() -> Result<Vec<u8>, HttpError> {
    let now_ms = ic_cdk::api::time() / 1_000_000;
    let dt = Utc.timestamp_millis_opt(now_ms as i64).unwrap();
    let tokens = store::state::list_uniswap_tokens();
    let body = json!({
        "name": "TokenList.ing",
        "timestamp": dt.to_rfc3339(),
        "logoURI": "https://tokenlist.ing/_assets/logo.webp",
        "version": {
          "major": 1,
          "minor": 0,
          "patch": 0
        },
        "tokens": tokens,
    });
    serde_json::to_vec(&body).map_err(|err| HttpError {
        status_code: 500,
        message: format!("failed to serialize tokens, error: {err}"),
    })
}

fn get_token(path: &str) -> Result<Vec<u8>, HttpError> {
    let path = path.trim_start_matches('/');
    let path = path.trim_end_matches(".json");
    let token = store::state::get_token_by_location(path).map_err(|err| HttpError {
        status_code: 404,
        message: err,
    })?;
    serde_json::to_vec(&token).map_err(|err| HttpError {
        status_code: 500,
        message: format!("failed to serialize tokens, error: {err}"),
    })
}
