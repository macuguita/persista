use std::time::Duration;

use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::error::AppError;
use crate::identifier::Identifier;
use crate::model::Entitlements;

pub async fn init_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(db_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

pub async fn get_value(
    pool: &PgPool,
    uuid: Uuid,
    id: Identifier,
) -> Result<Option<serde_json::Value>, AppError> {
    let value = sqlx::query_scalar!(
        r#"
        SELECT value
        FROM player_data
        WHERE uuid = $1 AND namespace = $2 AND path = $3
        "#,
        uuid,
        id.namespace,
        id.path
    )
    .fetch_optional(pool)
    .await?;

    Ok(value)
}

pub async fn fetch_entitlements(pool: &PgPool, uuid: Uuid) -> Result<Entitlements, AppError> {
    let key = crate::identifier::entitlements_key();
    let value = get_value(pool, uuid, key).await?;

    Ok(value
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(Entitlements::empty))
}

pub async fn upsert_value(
    pool: &PgPool,
    uuid: Uuid,
    id: Identifier,
    value: &serde_json::Value,
) -> Result<(), AppError> {
    let updated_at = chrono_now_millis();

    sqlx::query!(
        r#"
        INSERT INTO player_data (uuid, namespace, path, value, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (uuid, namespace, path)
        DO UPDATE SET value = EXCLUDED.value, updated_at = EXCLUDED.updated_at
        "#,
        uuid,
        id.namespace,
        id.path,
        value,
        i64::try_from(updated_at)?
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_all_for_player(pool: &PgPool, uuid: Uuid) -> Result<u64, AppError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM player_data
        WHERE uuid = $1
        "#,
        uuid
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

fn chrono_now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is before 1970")
        .as_millis()
}
