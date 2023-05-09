use axum::{
    extract::{Path, State},
    response::Json,
};
use serde_json::{Value};
use crate::catalog::command::add_book_cmd::{AddBookCommand, AddBookCommandRequest, AddBookCommandResponse};
use crate::catalog::command::get_book_cmd::{GetBookCommand, GetBookCommandRequest, GetBookCommandResponse};
use crate::catalog::command::remove_book_cmd::{RemoveBookCommand, RemoveBookCommandRequest, RemoveBookCommandResponse};
use crate::catalog::domain::CatalogService;
use crate::catalog::factory;
use crate::core::command::Command;
use crate::core::controller::{AppState, json_to_server_error, ServerError};
use crate::utils::ddb::{build_db_client, create_table};

async fn build_service(state: AppState) -> Box<dyn CatalogService> {
    let client = build_db_client(state.store).await;
    let _ = create_table(&client, "books", "book_id", "book_status", "isbn").await;
    factory::create_catalog_service(&state.config, state.store).await
}

pub(crate) async fn add_book(
    State(state): State<AppState>,
    json: Json<Value>) -> Result<Json<AddBookCommandResponse>, ServerError> {
    let req: AddBookCommandRequest = serde_json::from_value(json.0).map_err(json_to_server_error)?;
    let svc = build_service(state).await;
    let res = AddBookCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}

pub(crate) async fn find_book_by_id(
    State(state): State<AppState>,
    Path(book_id): Path<String>) -> Result<Json<GetBookCommandResponse>, ServerError> {
    let req = GetBookCommandRequest { book_id };
    let svc = build_service(state).await;
    let res = GetBookCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}

pub(crate) async fn remove_book(
    State(state): State<AppState>,
    Path(book_id): Path<String>) -> Result<Json<RemoveBookCommandResponse>, ServerError> {
    let req = RemoveBookCommandRequest { book_id };
    let svc = build_service(state).await;
    let res = RemoveBookCommand::new(svc).execute(req).await?;
    Ok(Json(res))
}
