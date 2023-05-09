use axum::{
    extract::State,
    response::Json,
};
use serde_json::{Value};
use crate::core::command::Command;
use crate::core::controller::{AppState, json_to_server_error, ServerError};
use crate::hold::command::cancel_hold_book_cmd::{CancelHoldBookCommand, CancelHoldBookCommandRequest, CancelHoldBookCommandResponse};
use crate::hold::command::checkout_hold_book_cmd::{CheckoutHoldBookCommand, CheckoutHoldBookCommandRequest, CheckoutHoldBookCommandResponse};
use crate::hold::command::hold_book_cmd::{HoldBookCommand, HoldBookCommandRequest, HoldBookCommandResponse};
use crate::hold::domain::HoldService;
use crate::hold::factory;
use crate::utils::ddb::{build_db_client, create_table};

async fn build_service(state: AppState) -> Box<dyn HoldService> {
    let client = build_db_client(state.store).await;
    let _ = create_table(&client, "hold", "hold_id", "hold_status", "patron_id").await;
    factory::create_hold_service(&state.config, state.store).await
}

pub(crate) async fn hold_book(
    State(state): State<AppState>,
    json: Json<Value>) -> Result<Json<HoldBookCommandResponse>, ServerError> {
    let req: HoldBookCommandRequest = serde_json::from_value(json.0).map_err(json_to_server_error)?;
    let svc = build_service(state).await;
    let res = HoldBookCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}

pub(crate) async fn checkout_hold(
    State(state): State<AppState>,
    json: Json<Value>) -> Result<Json<CheckoutHoldBookCommandResponse>, ServerError> {
    let req: CheckoutHoldBookCommandRequest = serde_json::from_value(json.0).map_err(json_to_server_error)?;
    let svc = build_service(state).await;
    let res = CheckoutHoldBookCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}

pub(crate) async fn cancel_hold(
    State(state): State<AppState>,
    json: Json<Value>) -> Result<Json<CancelHoldBookCommandResponse>, ServerError> {
    let req: CancelHoldBookCommandRequest = serde_json::from_value(json.0).map_err(json_to_server_error)?;
    let svc = build_service(state).await;
    let res = CancelHoldBookCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}
