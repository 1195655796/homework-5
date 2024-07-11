use crate::{AppError, AppState, CreateChat};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chat_core::User;

use chat_core::Chat;
use anyhow::Context;
use tracing::info;





#[utoipa::path(
    get,
    path = "/api/chats",
    responses(
        (status = 200, description = "List of chats", body = Vec<Chat>),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.fetch_chats(user.ws_id as _).await?;
    info!("Listed chats");
    Ok((StatusCode::OK, Json(chat)))
}





#[utoipa::path(
    post,
    path = "/api/chats",
    responses(
        (status = 201, description = "Chat created", body = Chat),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(input, user.ws_id as _).await?;
    info!("Chat created");
    Ok((StatusCode::CREATED, Json(chat)))
}
#[utoipa::path(
    get,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id")
    ),
    responses(
        (status = 200, description = "Chat found", body = Chat),
        (status = 404, description = "Chat not found", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id as _).await?;
    info!("Chat geted");
    match chat {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::NotFound(format!("chat id {id}"))),
    }
}

// Homework
#[utoipa::path(
    patch,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id")
    ),
    request_body = CreateChat,
    responses(
        (status = 200, description = "Chat updated", body = Chat),
        (status = 404, description = "Chat not found", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state
        .update_chat(
            id,
            input.clone(),
            user.ws_id as u64,
            input.members)
        .await?;
    info!("Chat updated");
    Ok((StatusCode::OK, Json(chat)))
}

#[utoipa::path(
    delete,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id")
    ),
    responses(
        (status = 204, description = "Chat deleted", body = Chat),
        (status = 404, description = "Chat not found", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn delete_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    //println!("Received delete request for chat id {:?}", id);
    state.delete_chat(id, user.ws_id as _).await?;
    info!("Chat deleted");
    Ok(StatusCode::NO_CONTENT)
}

impl AppState {
    pub async fn update_chat(
        &self,
        id: u64,
        input: CreateChat,
        ws_id: u64,
        members: Vec<i64>
    ) -> Result<Chat, AppError> {
        let chat: Chat = sqlx::query_as(
            r#"
            UPDATE chats
            SET name = $1,
            members = $2
            WHERE id = $3 AND ws_id = $4
            RETURNING id, name, ws_id, created_at, members, type
            "#
        )
        .bind(&input.name)
        .bind(&(members as Vec<i64>))
        .bind(&(id as i64))
        .bind(&(ws_id as i64))
        .fetch_one(&self.inner.pool)
        .await
        .context("update chat failed")?;

        Ok(chat)
    }


    pub async fn delete_chat(&self, id: u64, ws_id: u64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM chats
            WHERE id = $1 AND ws_id = $2
            "#
        )
        .bind(&(id as i64))
        .bind(&(ws_id as i64))
        .execute(&self.inner.pool)
        .await
        .context("delete chat failed")?;
        Ok(())
    }
}