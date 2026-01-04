//! This crate contains all shared fullstack server functions.
use dioxus::{fullstack::Json, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct EchoInput {
    pub message: String,
}

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo(Json(input): Json<EchoInput>) -> Result<EchoInput, ServerFnError> {
    use dioxus::logger::tracing;
    tracing::info!("Echoing message: {}", input.message);
    Ok(input)
}
