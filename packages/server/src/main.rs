use server::{
    db,
    routes::{
        auth::{
            authenticate_with_github::authenticate_with_github,
            authenticate_with_password::authenticate_with_password, create_account::create_account,
            get_profile::get_profile,
        },
        doc::{doc, openapi_spec_handler},
    },
    AppState,
};
use std::net::SocketAddr;
use tracing::info;

use axum::{
    routing::{get, post},
    Router,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_pool = db::connect_db()
        .await
        .expect("Couldn't connect to database");

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let github_client_id = std::env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
    let github_client_secret =
        std::env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
    let github_oauth_redirect_url =
        std::env::var("GITHUB_OAUTH_REDIRECT_URL").expect("GITHUB_OAUTH_REDIRECT_URL must be set");

    let app_state = AppState {
        db: db_pool,
        jwt_secret,
        github_client_id,
        github_client_secret,
        github_oauth_redirect_url,
    };

    let port: u16 = std::env::var("PORT")
        .unwrap_or("3000".into())
        .parse()
        .expect("PORT not found");

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/doc", get(doc))
        .route("/openapi.json", get(openapi_spec_handler))
        .route("/users", post(create_account))
        .route("/sessions/password", post(authenticate_with_password))
        .route("/sessions/github", post(authenticate_with_github))
        .route("/profile", get(get_profile))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server running in http://{}", addr);
    info!("Docs running in http://{}/doc", addr);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
