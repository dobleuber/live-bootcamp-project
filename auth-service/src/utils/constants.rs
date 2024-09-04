use dotenv::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;
use secrecy::Secret;

lazy_static! {
    pub static ref JWT_SECRET: Secret<String> = Secret::new(init_env_var(env::JWT_SECRET_ENV_VAR));
    pub static ref DATABASE_URL: Secret<String> = Secret::new(init_env_var(env::DATABASE_URL_ENV_VAR));
    pub static ref DATABASE_NAME: String = init_env_var(env::DATABASE_NAME_ENV_VAR);
    pub static ref REDIS_HOST_NAME: String = init_env_var_or_default(env::REDIS_HOST_NAME_ENV_VAR, DEFAULT_REDIS_HOSTNAME);
}

fn init_env_var(var_name: &str) -> String {
    dotenv().ok();
    let value = std_env::var(var_name).unwrap_or_else(|_| panic!("{} must be set.", var_name));

    if value.is_empty() {
        panic!("{} must not be empty", var_name);
    }

    value
}

fn init_env_var_or_default(var_name: &str, default_value: &str) -> String {
    dotenv().ok();
    std_env::var(var_name).unwrap_or_else(|_| default_value.to_string())
}


pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const DATABASE_NAME_ENV_VAR: &str = "DATABASE_NAME";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";


pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:8080";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}