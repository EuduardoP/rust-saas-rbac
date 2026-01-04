use server::routes::doc::ApiDoc;
use std::fs;
use utoipa::OpenApi;

fn main() {
    let doc = ApiDoc::openapi().to_pretty_json().unwrap();
    fs::write("openapi.json", doc).unwrap();
    println!("✅ OpenAPI specification generated to openapi.json");
}
