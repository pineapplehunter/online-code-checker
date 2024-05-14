use askama::Template;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use axum_messages::{Message, Messages};

use crate::users::AuthSession;

#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    messages: Vec<Message>,
    username: &'a str,
}

pub fn router() -> Router<()> {
    Router::new()
        .route("/", get(self::get::index))
        .route("/problem", get(self::get::problems))
        // .route("/problem/{id}", get(self::get::problem_info))
        // .route("/problem/{id}/submit", get(self::get::submit))
}

mod get {
    use super::*;

    pub async fn index(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => ProtectedTemplate {
                messages: messages.into_iter().collect(),
                username: &user.username,
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn problems(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => ProtectedTemplate {
                messages: messages.into_iter().collect(),
                username: &user.username,
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
