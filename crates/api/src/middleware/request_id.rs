use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn trace_request_id(mut req: Request, next: Next) -> Response {
    let request_id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(val) => match val.to_str() {
            Ok(v) if !v.trim().is_empty() => v.to_string(),
            _ => Uuid::new_v4().to_string(),
        },
        None => Uuid::new_v4().to_string(),
    };

    req.headers_mut().insert(
        REQUEST_ID_HEADER,
        request_id.parse().unwrap(),
    );

    let mut response = next.run(req).await;

    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        request_id.parse().unwrap(),
    );

    response
}