use dotenv::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

lazy_static! {
    pub static ref JWT_SECRET: String = init_env_var(env::JWT_SECRET_ENV_VAR, "JWT_SECRET");
    pub static ref DATABASE_URL: String = init_env_var(env::DATABASE_URL_ENV_VAR, "DATABASE_URL");
    pub static ref DATABASE_NAME: String = init_env_var(env::DATABASE_NAME_ENV_VAR, "DATABASE_NAME");
}

fn init_env_var(var_name: &str, env_label: &str) -> String {
    dotenv().ok();
    let value = std_env::var(var_name).unwrap_or_else(|_| panic!("{} must be set.", env_label));

    if value.is_empty() {
        panic!("{} must not be empty", env_label);
    }

    value
}


pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const DATABASE_NAME_ENV_VAR: &str = "DATABASE_NAME";
}

pub const JWT_COOKIE_NAME: &str = "jwt";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:8080";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}