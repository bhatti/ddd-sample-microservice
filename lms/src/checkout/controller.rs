use axum::{
    extract::State,
    response::Json,
};
use serde_json::{Value};
use crate::checkout::command::checkout_book_cmd::{CheckoutBookCommand, CheckoutBookCommandRequest, CheckoutBookCommandResponse};
use crate::checkout::command::return_book_cmd::{ReturnBookCommand, ReturnBookCommandRequest, ReturnBookCommandResponse};
use crate::checkout::domain::CheckoutService;
use crate::checkout::factory;
use crate::core::command::Command;
use crate::core::controller::{AppState, json_to_server_error, ServerError};
use crate::utils::ddb::{build_db_client, create_table};

async fn build_service(state: AppState) -> Box<dyn CheckoutService> {
    let client = build_db_client(state.store).await;
    let _ = create_table(&client, "checkout", "checkout_id", "checkout_status", "patron_id").await;
    factory::create_checkout_service(&state.config, state.store).await
}

pub(crate) async fn checkout_book(
    State(state): State<AppState>,
    json: Json<Value>) -> Result<Json<CheckoutBookCommandResponse>, ServerError> {
    let req: CheckoutBookCommandRequest = serde_json::from_value(json.0).map_err(json_to_server_error)?;
    let svc = build_service(state).await;
    let res = CheckoutBookCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}

pub(crate) async fn return_book(
    State(state): State<AppState>,
    json: Json<Value>) -> Result<Json<ReturnBookCommandResponse>, ServerError> {
    let req: ReturnBookCommandRequest = serde_json::from_value(json.0).map_err(json_to_server_error)?;
    let svc = build_service(state).await;
    let res = ReturnBookCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}
