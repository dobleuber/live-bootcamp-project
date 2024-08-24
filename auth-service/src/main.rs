use sqlx::mysql::MySqlPool;

use auth_service::{
    domain::IntoShared, services::data_stores::{
        hashmap_two_fa_code_store::HashmapTwoFACodeStore,
        my_sql_user_store::MySqlUserStore,
        hashset_banned_token_store::HashSetBannedTokenStore,
        mock_email_client::MockEmailClient,
    }, utils::constants::{prod, DATABASE_NAME, DATABASE_URL}, AppState, Application, get_mysql_pool,
};

#[tokio::main]
async fn main() {
    let db_pool = configure_database().await;
    let user_store = MySqlUserStore::new(db_pool).into_shared();
    let banned_token_store = HashSetBannedTokenStore::default().into_shared();
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
    let connection_string = format!("{}/{}", DATABASE_URL.to_string(), DATABASE_NAME.to_string());
    let db_pool = get_mysql_pool(&connection_string).await.expect("Failed to connect to MySQL");

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    db_pool
}