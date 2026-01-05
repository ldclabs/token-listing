use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use candid::CandidType;
use chrono::prelude::*;
use ic_auth_types::cbor_into_vec;
use ic_http_certification::{HeaderField, HttpRequest};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::{store, types};

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

    let ims = get_request_header(&request, "if-modified-since")
        .and_then(parse_http_date)
        .map(|dt| dt.timestamp_millis() as u64);

    let rt = match (request.method().as_str(), req_path.as_str()) {
        ("HEAD", _) => Ok(Vec::new()),
        ("GET", "/") => get_info(),
        ("GET", "/.well-known/ic-domains") => get_domains(),
        ("GET", path) => match get_image(path, ims) {
            Ok((metadata, body)) => {
                let last_modified_sec = (metadata.updated_at / 1000) as i64;
                let dt = Utc.timestamp_opt(last_modified_sec, 0).unwrap();
                headers.push((
                    "last-modified".to_string(),
                    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
                ));
                headers.push(("content-type".to_string(), metadata.r#type));
                headers.push(("content-disposition".to_string(), "inline".to_string()));
                headers.push((
                    "cache-control".to_string(),
                    "max-age=2592000, public".to_string(),
                ));

                if let Some(body) = body {
                    headers.push(("content-length".to_string(), metadata.size.to_string()));

                    return HttpResponse {
                        status_code: 200,
                        headers,
                        body: body.into(),
                        upgrade: None,
                    };
                }

                headers.push(("content-length".to_string(), "0".to_string()));
                return HttpResponse {
                    status_code: 304,
                    headers,
                    body: Vec::new().into(),
                    upgrade: None,
                };
            }
            Err(err) => Err(err),
        },
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

fn get_domains() -> Result<Vec<u8>, HttpError> {
    Ok(b"image.tokenlist.ing".to_vec())
}

fn get_image(
    path: &str,
    ims: Option<u64>,
) -> Result<(types::ImageMetadata, Option<Vec<u8>>), HttpError> {
    let path = path.trim_start_matches('/').to_ascii_lowercase();
    let path = path
        .trim_end_matches(".png")
        .trim_end_matches(".webp")
        .trim_end_matches(".svg");
    let (metadata, body) = store::state::get_image(&path, ims).ok_or_else(|| HttpError {
        status_code: 404,
        message: format!("Image not found for path: {}", path),
    })?;

    Ok((metadata, body))
}

fn get_request_header<'a>(request: &'a HttpRequest<'static>, name: &str) -> Option<&'a str> {
    request
        .headers()
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn parse_http_date(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
