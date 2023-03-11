use axum::Extension;
use sqlx::PgPool;

#[tracing::instrument]
pub(crate) async fn health_check(_: Extension<PgPool>) -> () {}
