use fleet_telemetry_api::{config::Config, db, logging, AppState};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    logging::init(config.log_json);
    let pool = db::connect(&config).await?;
    db::migrate(&pool).await?;
    db::bootstrap_admin(&pool, config.bootstrap_admin.as_ref()).await?;
    let addr: SocketAddr = config.bind_addr;
    let app = fleet_telemetry_api::build_router(AppState::new(pool, config));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "fleet telemetry API listening");
    axum::serve(listener, app).await?;
    Ok(())
}
