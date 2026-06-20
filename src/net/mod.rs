use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
};
use serde_json::Value;
use tokio::time::timeout;
use uuid::Uuid;

pub mod jwt;
pub mod model;
pub mod mojang;
pub mod rate_limit;

use crate::AppState;
use crate::db;
use crate::error::AppError;
use crate::identifier::Identifier;
use jwt::AuthUser;
use mojang::MojangAuth;
use rate_limit::RateLimiter;

pub fn setup_routes(state: Arc<AppState>) -> Router {
    let admin_routes = Router::new()
        .route("/entitlements/{uuid}", post(post_entitlements_admin))
        .route("/data/{uuid}", delete(delete_data))
        .route_layer(middleware::from_fn_with_state(state.clone(), admin_layer));

    let auth_routes = Router::new()
        .route("/api/auth/mojang/challenge", post(post_challenge))
        .route("/api/auth/mojang", post(post_auth))
        .route_layer(middleware::from_fn_with_state(
            state.auth_limiter.clone(),
            |State(limiter): State<Arc<RateLimiter>>, req: Request<Body>, next: Next| async move {
                rate_limit::rate_limit_middleware(limiter, req, next).await
            },
        ));

    let data_routes = Router::new()
        .route(
            "/api/v0/data/{uuid}/{namespace}/{path}",
            get(get_data).post(post_data),
        )
        .route_layer(middleware::from_fn_with_state(
            state.data_limiter.clone(),
            |State(limiter): State<Arc<RateLimiter>>, req: Request<Body>, next: Next| async move {
                rate_limit::rate_limit_middleware(limiter, req, next).await
            },
        ));

    Router::new()
        .merge(auth_routes)
        .merge(data_routes)
        .nest("/api/admin", admin_routes)
        .layer(middleware::from_fn(timeout_middleware))
        .layer(DefaultBodyLimit::max(1024))
        .with_state(state)
}

async fn get_data(
    Path((uuid, namespace, path)): Path<(Uuid, String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let id = Identifier::new(namespace, path)?;

    let value = db::get_value(&state.db_pool, uuid, id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(value))
}

async fn post_challenge(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<model::ChallengeRequest>,
) -> Result<Json<model::ChallengeResponse>, AppError> {
    if payload.id.is_empty() {
        return Err(AppError::BadRequest("is is required".to_string()));
    }

    let challenge = MojangAuth::generate_challenge();
    state.mojang.store_challenge(&payload.id, challenge.clone());

    Ok(Json(model::ChallengeResponse {
        token: challenge,
        expires_in: 30,
    }))
}

async fn post_auth(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<model::AuthRequest>,
) -> Result<Json<model::SessionResponse>, AppError> {
    let stored = state
        .mojang
        .consume_challenge(&payload.id)
        .ok_or_else(|| AppError::Unauthorized("challenge not found or expired".to_string()))?;

    if stored != payload.token {
        return Err(AppError::Unauthorized(
            "challenge token mismatch".to_string(),
        ));
    }

    let profile = state
        .mojang
        .verify_with_mojang(&payload.username, &stored)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Mojang verification failed".to_string()))?;

    let mojang_id = profile.id.replace('-', "");
    let claimed_id = payload.id.replace('-', "");
    if mojang_id != claimed_id {
        return Err(AppError::Unauthorized("UUID mismatch".to_string()));
    }

    let uuid_str = if profile.id.contains('-') {
        profile.id.clone()
    } else {
        let id = &profile.id;
        format!(
            "{}-{}-{}-{}-{}",
            &id[..8],
            &id[8..12],
            &id[12..16],
            &id[16..20],
            &id[20..]
        )
    };

    let user_id = Uuid::parse_str(&uuid_str)?;

    let session = jwt::mint(&state.config.jwt_secret, user_id)?;

    Ok(Json(model::SessionResponse {
        user_id: session.user_id.to_string(),
        token: session.token,
        expires_at: session.expires_at,
    }))
}

async fn post_data(
    Path((uuid, namespace, path)): Path<(Uuid, String, String)>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    let data_id = Identifier::new(&namespace, &path)?;

    if data_id == crate::identifier::entitlements_key() {
        return Err(AppError::Unauthorized(
            "entitlements are managed server-side only".to_string(),
        ));
    }

    if auth.user_id != uuid {
        return Err(AppError::Unauthorized(
            "cannot write data for another player".to_string(),
        ));
    }

    let entitlements = db::fetch_entitlements(&state.db_pool, uuid).await?;
    if !entitlements.contains(&data_id) {
        return Err(AppError::Unauthorized(format!(
            "missing entitlement: {data_id}"
        )));
    }

    db::upsert_value(&state.db_pool, uuid, data_id, &body).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn post_entitlements_admin(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<model::Entitlements>,
) -> Result<(), AppError> {
    payload.validate()?;
    let json = serde_json::json!(&payload);

    db::upsert_value(
        &state.db_pool,
        uuid,
        crate::identifier::entitlements_key(),
        &json,
    )
    .await?;

    Ok(())
}

async fn delete_data(
    Path(uuid): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<(), AppError> {
    db::delete_all_for_player(&state.db_pool, uuid).await?;

    Ok(())
}

async fn admin_layer(
    State(state): State<Arc<AppState>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let secret = req
        .headers()
        .get("X-Admin-Secret")
        .and_then(|v| v.to_str().ok());

    if secret != Some(&state.config.admin_secret) {
        return Err(AppError::Unauthorized("not authorized".to_string()));
    }

    Ok(next.run(req).await)
}

async fn timeout_middleware(req: Request<Body>, next: Next) -> Result<Response, AppError> {
    Ok(timeout(Duration::from_secs(30), next.run(req)).await?)
}
