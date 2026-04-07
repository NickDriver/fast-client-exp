use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tower_sessions::Session;
use uuid::Uuid;

pub const USER_ID_KEY: &str = "user_id";

pub async fn require_auth(
    session: Session,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let user_id: Option<Uuid> = session.get(USER_ID_KEY).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    match user_id {
        Some(id) => {
            req.extensions_mut().insert(id);
            Ok(next.run(req).await)
        }
        None => Err(StatusCode::SEE_OTHER),
    }
}

pub async fn store_user_session(session: &Session, user_id: Uuid) -> Result<(), StatusCode> {
    session
        .insert(USER_ID_KEY, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn clear_session(session: &Session) -> Result<(), StatusCode> {
    session
        .flush()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
