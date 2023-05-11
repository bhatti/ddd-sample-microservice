include!("../../lib.rs");
use axum::{
    routing::{get, post},
    Router,
};
use lambda_http::{run, Error};
use crate::utils::ddb::setup_tracing;
use crate::core::controller::AppState;
use crate::core::repository::RepositoryStore;
use crate::hold::controller::{hold_book, cancel_hold, checkout_hold};

const DEV_MODE: bool = true;

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_tracing();

    let state = if DEV_MODE {
        std::env::set_var("AWS_LAMBDA_FUNCTION_NAME", "_");
        std::env::set_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE", "4096"); // 200MB
        std::env::set_var("AWS_LAMBDA_FUNCTION_VERSION", "1");
        std::env::set_var("AWS_LAMBDA_RUNTIME_API", "http://[::]:9000/.rt");
        AppState::new("dev", RepositoryStore::LocalDynamoDB)
    } else {
        AppState::new("prod", RepositoryStore::DynamoDB)
    };

    let app = Router::new()
        .route("/hold", post(hold_book))
        .route("/hold/checkout", post(checkout_hold))
        .route("/hold/cancel", post(cancel_hold))
        .with_state(state);

    run(app).await
}
