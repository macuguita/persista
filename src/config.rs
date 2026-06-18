use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub jwt_secret: String,
    pub db_url: String,
    pub admin_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("ERROR: Port must be a valid number"),
            jwt_secret: required_env("JWT_SECRET"),
            db_url: required_env("DB_URL"),
            admin_secret: required_env("ADMIN_SECRET"),
        }
    }
}

fn required_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("missing required env var: {key}"))
}
