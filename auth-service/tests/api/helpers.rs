use std::sync::Arc;

use auth_service::{
    Application,
    AppState,
    BannedTokenStoreType,
    TwoFACodeStoreType,
    services::{
        hashmap_user_store::HashmapUserStore,
        hashset_banned_token_store::HashSetBannedTokenStore,
        hashmap_two_fa_code_store::HashmapTwoFACodeStore,
    },
    domain::IntoShared,
    utils::constants::test,
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
        let user_store = HashmapUserStore::default().into_shared();
        let banned_token_store = HashSetBannedTokenStore::default().into_shared();
        let two_fa_code_store = HashmapTwoFACodeStore::default().into_shared();
        let app_state = AppState::new(user_store, banned_token_store.clone(), two_fa_code_store.clone());
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
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
