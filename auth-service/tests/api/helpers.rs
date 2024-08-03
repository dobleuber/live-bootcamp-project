use auth_service::{
    Application,
    AppState,
    services::hashmap_user_store::HashmapUserStore,
    domain::UserStore
};
use reqwest::Client;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::default().into_shared();
        let app_state = AppState::new(user_store);
        let app = Application::build(app_state, "127.0.0.1:0")
            .await
            .expect("Failed to build the app");

        let address = format!("http://{}", &app.address);

        let _app = tokio::spawn(app.run());

        let http_client = Client::new();

        Self {
            address,
            http_client,
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

    pub async fn get_login(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_logout(&self) -> reqwest::Response {
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

    pub async fn get_verify_token(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_account<Body>(&self, body: &Body) -> reqwest::Response
    where Body: serde::Serialize
    {
        self.http_client
            .post(&format!("{}/delete-account", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
