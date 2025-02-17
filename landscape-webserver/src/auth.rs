use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use axum::http::header::AUTHORIZATION;
use axum::Json;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::TokenData;
use landscape_common::args::LAND_ARGS;
use once_cell::sync::Lazy;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};

const SECRET_KEY_LENGTH: usize = 20;

pub static SECRET_KEY: Lazy<String> = Lazy::new(|| {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(SECRET_KEY_LENGTH)
        .map(char::from)
        .collect()
});

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // 用户ID或标识
    sub: String,
    // 过期时间（Unix timestamp）
    exp: usize,
}

fn create_jwt(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    // 设置过期时间（例如1小时后）
    let expiration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize + 3600;
    let claims = Claims { sub: user_id.to_owned(), exp: expiration };
    // 使用一个足够复杂的密钥来签名
    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY.as_bytes()))
}

pub(crate) async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION.as_str())
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response())?;

    // 期望格式为 "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization header format").into_response())?;

    // 验证并解码 token
    let token_data: TokenData<Claims> =
        decode(token, &DecodingKey::from_secret(SECRET_KEY.as_bytes()), &Validation::default())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token").into_response())?;

    if token_data.claims.sub == LAND_ARGS.admin_user {
        Ok(next.run(req).await)
    } else {
        Err((StatusCode::UNAUTHORIZED, "Invalid token").into_response())
    }
}

pub async fn login_handler(
    Json(LoginInfo { username, password }): Json<LoginInfo>,
) -> Result<impl IntoResponse, Response> {
    let mut result = LoginResult { success: false, token: "".to_string() };
    if username == LAND_ARGS.admin_user && password == LAND_ARGS.admin_pass {
        result.success = true;
        result.token = create_jwt(&username).unwrap();
    } else {
        return Err((StatusCode::UNAUTHORIZED, "Invalid username or password").into_response());
    }
    Ok(Json(result))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginInfo {
    username: String,
    password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginResult {
    success: bool,
    token: String,
}
