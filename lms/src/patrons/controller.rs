use axum::{
    extract::{Path, State},
    response::Json,
};
use serde_json::{Value};
use crate::core::command::Command;
use crate::core::controller::{AppState, json_to_server_error, ServerError};
use crate::patrons::command::add_patron_cmd::{AddPatronCommand, AddPatronCommandRequest, AddPatronCommandResponse};
use crate::patrons::command::get_patron_cmd::{GetPatronCommand, GetPatronCommandRequest, GetPatronCommandResponse};
use crate::patrons::command::remove_patron_cmd::{RemovePatronCommand, RemovePatronCommandRequest, RemovePatronCommandResponse};
use crate::patrons::domain::PatronService;
use crate::patrons::factory;
use crate::utils::ddb::{build_db_client, create_table};

async fn build_service(state: AppState) -> Box<dyn PatronService> {
    let client = build_db_client(state.store).await;
    let _ = create_table(&client, "parties", "party_id", "kind", "email").await;
    factory::create_patron_service(&state.config, state.store).await
}

pub(crate) async fn add_patron(
    State(state): State<AppState>,
    json: Json<Value>) -> Result<Json<AddPatronCommandResponse>, ServerError> {
    let req: AddPatronCommandRequest = serde_json::from_value(json.0).map_err(json_to_server_error)?;
    let svc = build_service(state).await;
    let res = AddPatronCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}

pub(crate) async fn find_patron_by_id(
    State(state): State<AppState>,
    Path(patron_id): Path<String>) -> Result<Json<GetPatronCommandResponse>, ServerError> {
    let req = GetPatronCommandRequest { patron_id };
    let svc = build_service(state).await;
    let res = GetPatronCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}

pub(crate) async fn remove_patron(
    State(state): State<AppState>,
    Path(patron_id): Path<String>) -> Result<Json<RemovePatronCommandResponse>, ServerError> {
    let req = RemovePatronCommandRequest { patron_id };
    let svc = factory::create_patron_service(&state.config, state.store).await;
    let res = RemovePatronCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}