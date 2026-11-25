use tracing_subscriber::{fmt, prelude::*, EnvFilter};
pub fn init(json: bool) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,sqlx=warn"));
    let registry = tracing_subscriber::registry().with(filter);
    let result = if json {
        registry.with(fmt::layer().json().with_current_span(true).with_span_list(false)).try_init()
    } else { registry.with(fmt::layer()).try_init() };
    let _ = result;
}
