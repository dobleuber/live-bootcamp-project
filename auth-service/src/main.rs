use auth_service::{
    domain::IntoShared, services::{
        hashmap_two_fa_code_store::HashmapTwoFACodeStore,
        hashmap_user_store::HashmapUserStore,
        hashset_banned_token_store::HashSetBannedTokenStore,
        mock_email_client::MockEmailClient,
    }, utils::constants::prod, AppState, Application
};

#[tokio::main]
async fn main() {
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
