use sqlx::mysql::MySqlPool;

use auth_service::{
    domain::IntoShared, services::{
        hashmap_two_fa_code_store::HashmapTwoFACodeStore,
        hashmap_user_store::HashmapUserStore,
        hashset_banned_token_store::HashSetBannedTokenStore,
        mock_email_client::MockEmailClient,
    }, utils::constants::{prod, DATABASE_URL}, AppState, Application, get_mysql_pool,
};

#[tokio::main]
async fn main() {
    let _pg_pool = configure_database().await;
    let user_store = HashmapUserStore::default().into_shared();
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
    println!("{}", &DATABASE_URL.as_str());
    let db_pool = get_mysql_pool(&DATABASE_URL).await.expect("Failed to connect to MySQL");

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    db_pool
}