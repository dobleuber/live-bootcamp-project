use secrecy::{Secret, ExposeSecret};
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlConnection, MySqlPool, MySqlPoolOptions},
    Connection, Executor,
};
use tokio::sync::RwLock;
use std::{str::FromStr, sync::Arc};
use reqwest::cookie::Jar;
use uuid::Uuid;

use auth_service::{
    domain::IntoShared,
    get_mysql_pool,
    configure_redis,
    services::data_stores::{
        redis_two_fa_code_store::RedisTwoFACodeStore,
        redis_banned_token_store::RedisBannedTokenStore,
        mock_email_client::MockEmailClient,
        my_sql_user_store::MySqlUserStore,
    },
    utils::constants::{test, DATABASE_URL, DEFAULT_REDIS_HOSTNAME},
    AppState, Application, BannedTokenStoreType, TwoFACodeStoreType,
};

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub db_name: String,
    pub clean_up_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (db_pool, db_name) = configure_my_sql().await;
        let redis_conn = Arc::new(RwLock::new(configure_redis(DEFAULT_REDIS_HOSTNAME.to_string())));
        let user_store = MySqlUserStore::new(db_pool).into_shared();
        let banned_token_store = RedisBannedTokenStore::new(redis_conn.clone()).into_shared();
        let two_fa_code_store = RedisTwoFACodeStore::new(redis_conn.clone()).into_shared();
        let mock_email_client = MockEmailClient.into_shared();
        let app_state = AppState::new(
            user_store,
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            mock_email_client,
        );
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build the app");

        let address = format!("http://{}", &app.address);

        let _app = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("Failed to build HTTP client");

        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            db_name,
            clean_up_called: false,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_account(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/delete-account", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn clean_up(&mut self) {
        if self.clean_up_called {
            return;
        }

        delete_database(&self.db_name).await;

        self.clean_up_called = true;
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("The clean app wasn't called before dropping the test");
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

async fn configure_my_sql() -> (MySqlPool, String) {
    let mysql_conn_url = DATABASE_URL.expose_secret().to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&mysql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", mysql_conn_url, db_name);

    // Create a new connection pool and return it
    (get_mysql_pool(Secret::new(postgresql_conn_url_with_db))
        .await
        .expect("Failed to create MySql connection pool!"), db_name)
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    println!("{}", db_conn_string);
    let connection = MySqlPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create MySQL connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE `{}`;"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = MySqlPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let db_conn_string: String = DATABASE_URL.expose_secret().to_owned();

    let connection_options = MySqlConnectOptions::from_str(&db_conn_string)
        .expect("Failed to parse the connection string");

    let mut connection = MySqlConnection::connect_with(&connection_options)
        .await
        .expect("failed to connect to MySQL");

    connection
        .execute(
            format!(
                r#"
SELECT 
    CONCAT('KILL ', id, ';') 
FROM 
    information_schema.processlist 
WHERE 
    db = '{}'
    AND id <> CONNECTION_ID();
 
            "#,
                &db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database");

    connection
        .execute(format!("DROP DATABASE `{}`;", db_name).as_str())
        .await
        .expect("Failed to drop the database");
}
