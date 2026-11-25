use std::{fmt, net::SocketAddr, str::FromStr};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    Missing(&'static str),
    #[error("invalid value for {name}: {reason}")]
    Invalid { name: &'static str, reason: String },
}
#[derive(Clone)]
pub struct BootstrapAdmin { pub email: String, pub password: String }
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub db_max_connections: u32,
    pub max_batch_size: usize,
    pub log_json: bool,
    pub bootstrap_admin: Option<BootstrapAdmin>,
}
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config").field("database_url", &"<redacted>")
            .field("bind_addr", &self.bind_addr).field("jwt_secret", &"<redacted>")
            .field("jwt_ttl_seconds", &self.jwt_ttl_seconds)
            .field("db_max_connections", &self.db_max_connections)
            .field("max_batch_size", &self.max_batch_size).field("log_json", &self.log_json)
            .field("bootstrap_admin", &self.bootstrap_admin.as_ref().map(|a| &a.email)).finish()
    }
}
impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|k| std::env::var(k).ok())
    }
    pub fn from_lookup<F: Fn(&str) -> Option<String>>(get: F) -> Result<Self, ConfigError> {
        let database_url = get("DATABASE_URL").ok_or(ConfigError::Missing("DATABASE_URL"))?;
        let jwt_secret = get("JWT_SECRET").ok_or(ConfigError::Missing("JWT_SECRET"))?;
        if jwt_secret.len() < 32 {
            return Err(ConfigError::Invalid { name: "JWT_SECRET", reason: "must be at least 32 characters".into() });
        }
        let bootstrap_admin = match (get("BOOTSTRAP_ADMIN_EMAIL"), get("BOOTSTRAP_ADMIN_PASSWORD")) {
            (Some(email), Some(password)) if password.len() >= 12 => Some(BootstrapAdmin { email, password }),
            (Some(_), Some(_)) => return Err(ConfigError::Invalid { name: "BOOTSTRAP_ADMIN_PASSWORD", reason: "must be at least 12 characters".into() }),
            (None, None) => None,
            _ => return Err(ConfigError::Invalid { name: "BOOTSTRAP_ADMIN_EMAIL", reason: "set both bootstrap admin variables or neither".into() }),
        };
        let jwt_ttl_seconds = parse_or(&get, "JWT_TTL_SECONDS", 3600)?;
        let db_max_connections = parse_or(&get, "DB_MAX_CONNECTIONS", 10)?;
        let max_batch_size = parse_or(&get, "MAX_BATCH_SIZE", 1000)?;
        if jwt_ttl_seconds <= 0 {
            return Err(ConfigError::Invalid { name: "JWT_TTL_SECONDS", reason: "must be greater than zero".into() });
        }
        if db_max_connections == 0 {
            return Err(ConfigError::Invalid { name: "DB_MAX_CONNECTIONS", reason: "must be greater than zero".into() });
        }
        if max_batch_size == 0 {
            return Err(ConfigError::Invalid { name: "MAX_BATCH_SIZE", reason: "must be greater than zero".into() });
        }
        Ok(Self {
            database_url,
            bind_addr: parse_or(&get, "BIND_ADDR", "0.0.0.0:8080".parse().unwrap())?,
            jwt_secret,
            jwt_ttl_seconds,
            db_max_connections,
            max_batch_size,
            log_json: get("LOG_FORMAT").is_some_and(|v| v.eq_ignore_ascii_case("json")),
            bootstrap_admin,
        })
    }
}
fn parse_or<T: FromStr, F: Fn(&str) -> Option<String>>(get: &F, name: &'static str, default: T) -> Result<T, ConfigError>
where T::Err: fmt::Display {
    match get(name) {
        None => Ok(default),
        Some(raw) => raw.trim().parse().map_err(|e: T::Err| ConfigError::Invalid { name, reason: e.to_string() }),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    const SECRET: &str = "0123456789abcdef0123456789abcdef";
    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String,String> = pairs.iter().map(|(k,v)| (k.to_string(),v.to_string())).collect();
        move |key| map.get(key).cloned()
    }
    #[test] fn defaults_load() {
        let c = Config::from_lookup(lookup(&[("DATABASE_URL","postgres://x"),("JWT_SECRET",SECRET)])).unwrap();
        assert_eq!(c.bind_addr.port(),8080); assert_eq!(c.max_batch_size,1000);
    }
    #[test] fn secrets_are_redacted() {
        let c = Config::from_lookup(lookup(&[("DATABASE_URL","postgres://user:pw@host/db"),("JWT_SECRET",SECRET)])).unwrap();
        let d = format!("{c:?}"); assert!(!d.contains(SECRET)); assert!(!d.contains("user:pw"));
    }
    #[test] fn rejects_short_secret() {
        assert!(Config::from_lookup(lookup(&[("DATABASE_URL","x"),("JWT_SECRET","short")])).is_err());
    }
}
