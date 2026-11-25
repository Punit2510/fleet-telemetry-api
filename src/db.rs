use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::{config::{BootstrapAdmin, Config}, auth::hash_password};

pub async fn connect(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new().max_connections(config.db_max_connections).connect(&config.database_url).await
}
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
pub async fn bootstrap_admin(pool: &PgPool, admin: Option<&BootstrapAdmin>) -> anyhow::Result<()> {
    if let Some(admin) = admin {
        let password_hash = hash_password(&admin.password)?;
        sqlx::query("INSERT INTO users (email, password_hash, role) VALUES ($1, $2, 'admin') ON CONFLICT (email) DO NOTHING")
            .bind(&admin.email).bind(password_hash).execute(pool).await?;
    }
    Ok(())
}
