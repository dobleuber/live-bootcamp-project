use auth_service::{Application, AppState, UserStoreType};

#[tokio::main]
async fn main() {
    let user_store = UserStoreType::default();
    let app_state = AppState::new(user_store);
    let app = Application::build(app_state, "0.0.0.0:8080")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
