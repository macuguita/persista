use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::DateTime;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

const ISSUER: &str = "persista";
const TTL_SECS: u64 = 24 * 60 * 60;

pub struct AuthUser {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub exp: u64,
    pub iss: String,
}

pub struct SessionToken {
    pub user_id: Uuid,
    pub session_token: String,
    pub expires_at: String,
}

pub fn mint(secret: &str, user_id: Uuid) -> Result<SessionToken, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970")
        .as_secs();

    let exp = now + TTL_SECS;

    let claims = Claims {
        user_id: user_id.to_string(),
        exp,
        iss: ISSUER.to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    );

    let expires_at = DateTime::from_timestamp(exp as i64, 0)
        .unwrap()
        .to_rfc3339();

    Ok(SessionToken {
        user_id,
        session_token: token?,
        expires_at,
    })
}

pub fn verify(secret: &str, token: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::default();
    validation.set_issuer(&[ISSUER]);

    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|x| x.claims)?)
}

impl FromRequestParts<Arc<crate::AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<crate::AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing Authorization header".to_string()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("invalid Authorization format".to_string()))?;

        let claims = verify(&state.config.jwt_secret, token)?;

        let user_id = Uuid::parse_str(&claims.user_id)?;

        Ok(AuthUser { user_id })
    }
}
