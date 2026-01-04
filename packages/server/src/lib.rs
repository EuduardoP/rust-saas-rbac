use sea_orm::DatabaseConnection;

pub mod db;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: String,
}