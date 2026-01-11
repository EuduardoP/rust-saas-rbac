use sea_orm::DatabaseConnection;

pub mod db;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_oauth_redirect_url: String,
}

