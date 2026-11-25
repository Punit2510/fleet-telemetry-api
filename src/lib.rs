pub mod alerts;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod logging;
pub mod models;
pub mod routes;
pub mod state;
pub mod validation;

pub use routes::build_router;
pub use state::AppState;
