use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use axum::extract::State;
use axum::routing::post;
use axum::Json;
use axum::Router;
use axum::{extract::Request, middleware::Next, response::Response};
use landscape_common::args::LAND_HOME_PATH;
use landscape_common::config::AuthRuntimeConfig;
use landscape_common::LANDSCAPE_SYS_TOKEN_FILE_ANME;
use once_cell::sync::Lazy;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::api::LandscapeApiResp;
use crate::auth::error::AuthError;
use crate::error::LandscapeApiError;
use crate::error::LandscapeApiResult;

pub mod error;

const SECRET_KEY_LENGTH: usize = 20;
const DEFAULT_EXPIRE_TIME: usize = 60 * 60 * 1;
const SYS_TOKEN_EXPIRE_TIME: usize = 60 * 60 * 24 * 365 * 30;

pub static SECRET_KEY: Lazy<String> = Lazy::new(|| {
    //
    rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(SECRET_KEY_LENGTH)
        .map(char::from)
        .collect()
});

pub async fn output_sys_token(auth: &AuthRuntimeConfig) {
    let token_path = LAND_HOME_PATH.join(LANDSCAPE_SYS_TOKEN_FILE_ANME);
    // 生成长期有效的系统 token
    let sys_token =
        create_jwt(&auth.admin_user, SYS_TOKEN_EXPIRE_TIME).expect("Failed to create system token");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(token_path)
        .await
        .expect("Failed to open landscape_api_token");

    // 写入系统 token
    file.write(sys_token.as_bytes()).await.expect("Failed to write system token");
    file.flush().await.expect("Failed to flush system token");
    // 设置文件权限为 0o400（仅文件所有者可读）
    let perms = Permissions::from_mode(0o400);
    file.set_permissions(perms).await.expect("Failed to set file permissions");
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // 用户ID或标识
    sub: String,
    // 过期时间（Unix timestamp）
    exp: usize,
}

fn create_jwt(user_id: &str, expiration: usize) -> Result<String, AuthError> {
    // 设置过期时间
    let expiration =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize + expiration;
    let claims = Claims { sub: user_id.to_owned(), exp: expiration };
    // 使用一个足够复杂的密钥来签名
    Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET_KEY.as_bytes()))?)
}

pub async fn auth_handler(
    State(auth): State<Arc<AuthRuntimeConfig>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, LandscapeApiError> {
    let Some(auth_header) =
        req.headers().get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok())
    else {
        return Err(AuthError::MissingAuthorizationHeader)?;
    };

    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return Err(AuthError::InvalidAuthorizationHeaderFormat)?;
    };

    let Ok(token_data) = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY.as_bytes()),
        &Validation::default(),
    ) else {
        return Err(AuthError::InvalidToken)?;
    };

    if token_data.claims.sub == auth.admin_user {
        Ok(next.run(req).await)
    } else {
        Err(AuthError::UnauthorizedUser)?
    }
}

pub fn get_auth_route(auth: Arc<AuthRuntimeConfig>) -> Router {
    Router::new().route("/login", post(login_handler)).with_state(auth)
}

async fn login_handler(
    State(auth): State<Arc<AuthRuntimeConfig>>,
    Json(LoginInfo { username, password }): Json<LoginInfo>,
) -> LandscapeApiResult<LoginResult> {
    let mut result = LoginResult { success: false, token: "".to_string() };
    if username == auth.admin_user && password == auth.admin_pass {
        result.success = true;
        result.token = create_jwt(&username, DEFAULT_EXPIRE_TIME)?;
    } else {
        return Err(AuthError::InvalidUsernameOrPassword)?;
    }
    LandscapeApiResp::success(result)
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
