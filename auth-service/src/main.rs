use std::sync::Arc;
use tokio::sync::RwLock;

use sqlx::mysql::MySqlPool;

use auth_service::{
    domain::IntoShared, services::data_stores::{
        hashmap_two_fa_code_store::HashmapTwoFACodeStore,
        my_sql_user_store::MySqlUserStore,
        redis_banned_token_store::RedisBannedTokenStore,
        mock_email_client::MockEmailClient,
    }, utils::constants::{prod, DATABASE_NAME, DATABASE_URL, REDIS_HOST_NAME}, AppState, Application, get_mysql_pool, get_redis_client
};

#[tokio::main]
async fn main() {
    let db_pool = configure_database().await;
    let redis_client = Arc::new(RwLock::new(configure_redis()));
    let user_store = MySqlUserStore::new(db_pool).into_shared();
    let banned_token_store = RedisBannedTokenStore::new(redis_client.clone()).into_shared();
    let hashmap_two_fa_code_store = HashmapTwoFACodeStore::default().into_shared();
    let mock_email_client = MockEmailClient.into_shared();
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        hashmap_two_fa_code_store,
        mock_email_client,
    );
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}


async fn configure_database() -> MySqlPool {
    let connection_string = format!("{}/{}", DATABASE_URL.as_str(), DATABASE_NAME.as_str());
    println!("Connection string: {}", &connection_string);
    let db_pool = get_mysql_pool(&connection_string).await.expect("Failed to connect to MySQL");

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    db_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}