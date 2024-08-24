use std::sync::Arc;
use sqlx::{mysql::{MySqlPool, MySqlPoolOptions}, Executor};

use auth_service::{
    domain::IntoShared, services::data_stores::{
        hashmap_two_fa_code_store::HashmapTwoFACodeStore,
        my_sql_user_store::MySqlUserStore,
        hashset_banned_token_store::HashSetBannedTokenStore,
        mock_email_client::MockEmailClient,
    }, utils::constants::{test, DATABASE_URL}, AppState, Application, BannedTokenStoreType, TwoFACodeStoreType, get_mysql_pool
};
use uuid::Uuid;
use reqwest::cookie::Jar;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType, 
}

impl TestApp {
    pub async fn new() -> Self {
        let db_pool = configure_my_sql().await;
        let user_store = MySqlUserStore::new(db_pool).into_shared();
        let banned_token_store = HashSetBannedTokenStore::default().into_shared();
        let two_fa_code_store = HashmapTwoFACodeStore::default().into_shared();
        let mock_email_client = MockEmailClient.into_shared();
        let app_state = AppState::new(user_store, banned_token_store.clone(), two_fa_code_store.clone(), mock_email_client);
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
    where Body: serde::Serialize
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

    pub async fn delete_account(&self) -> reqwest::Response
    {
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
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

async fn configure_my_sql() -> MySqlPool {
    let mysql_conn_url = DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&mysql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", mysql_conn_url, db_name);

    // Create a new connection pool and return it
    get_mysql_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create MySql connection pool!")
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
