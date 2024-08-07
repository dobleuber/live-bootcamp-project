use auth_service::{
    Application,
    AppState,
    services::{
        hashmap_user_store::HashmapUserStore,
        hashset_banned_token_store::HashSetBannedTokenStore,
    },
    domain::IntoShared,
    utils::constants::prod,
};

#[tokio::main]
async fn main() {
    let user_store = HashmapUserStore::default().into_shared();
    let banned_token_store = HashSetBannedTokenStore::default().into_shared();
    let app_state = AppState::new(user_store, banned_token_store);
    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
