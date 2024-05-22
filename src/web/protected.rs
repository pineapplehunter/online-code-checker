use crate::{config::Task, problems::ProblemsInfo, users::AuthSession};
use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_messages::{Message, Messages};
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::sync::mpsc::Sender;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    messages: Vec<Message>,
    username: &'a str,
}

#[derive(Template)]
#[template(path = "problems.html")]
struct ProblemsTemplate<'a> {
    messages: Vec<Message>,
    problems_info: ProblemsInfo,
    username: &'a str,
}

#[derive(Template)]
#[template(path = "solve.html")]
struct SolveTemplate<'a> {
    messages: Vec<Message>,
    index_html: String,
    username: &'a str,
}

#[derive(Template)]
#[template(path = "badge.html")]
struct BadgeTemplate {
    text: String,
    color: String,
    should_refresh: bool,
    problem_id: String,
}

#[derive(Template)]
#[template(path = "solution_badge.html")]
struct SolutionBadgeTemplate {
    text: String,
    color: String,
    should_refresh: bool,
    solution_id: String,
}

#[derive(Clone, Deserialize)]
struct Solution {
    id: i64,
    stdout: Option<String>,
    stderr: Option<String>,
    should_refresh: bool,
}

#[derive(Template)]
#[template(path = "status.html")]
struct StatusTemplate {
    messages: Vec<Message>,
    username: String,
    solutions: Vec<Solution>,
    problem_id: String,
}

#[derive(Clone, Debug)]
pub struct ServerState {
    db: SqlitePool,
    tx: Sender<Task>,
}

pub fn router(db: SqlitePool, tx: Sender<Task>) -> Router<()> {
    Router::new()
        .route("/", get(self::get::index))
        .route("/problems", get(self::get::problems))
        .route("/problem/:id/solve", get(self::get::solve))
        .route("/problem/:id/solve", post(self::post::solve))
        .route("/problem/:id/status", get(self::get::status))
        .route("/problem/:id/badge", get(self::get::badge))
        .route("/solution/:id/badge", get(self::get::solution_badge))
        // .route("/solution/:id/output", get(self::get::solution_output))
        .with_state(ServerState { db, tx })
        .fallback(|| async { Redirect::to("/") })
}

mod get {
    use axum::extract::State;

    use crate::problems::Problem;

    use super::*;

    pub async fn index(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => IndexTemplate {
                messages: messages.into_iter().collect(),
                username: &user.username,
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn problems(auth_session: AuthSession, messages: Messages) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => ProblemsTemplate {
                messages: messages.into_iter().collect(),
                problems_info: ProblemsInfo::get_cached_problems_info()
                    .await
                    .unwrap()
                    .clone(),
                username: &user.username,
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn solve(
        auth_session: AuthSession,
        messages: Messages,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => SolveTemplate {
                messages: messages.into_iter().collect(),
                username: &user.username,
                index_html: markdown::to_html(
                    &Problem::by_id(&id)
                        .await
                        .unwrap()
                        .get_index_md()
                        .await
                        .unwrap(),
                ),
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn badge(
        auth_session: AuthSession,
        State(state): State<ServerState>,
        Path(problem_id): Path<String>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let texts = sqlx::query!(
                    "select status from solutions where userid = ? and problem_id = ?",
                    user.id,
                    problem_id
                )
                .fetch_all(&state.db)
                .await
                .unwrap();
                if texts.is_empty() {
                    BadgeTemplate {
                        text: "未完了".to_string(),
                        color: "text-bg-light".to_string(),
                        should_refresh: false,
                        problem_id,
                    }
                } else if texts
                    .iter()
                    .flat_map(|s| s.status.clone())
                    .any(|s| s.to_lowercase() == "ac")
                {
                    BadgeTemplate {
                        text: "完了".to_string(),
                        color: "text-bg-success".to_string(),
                        should_refresh: false,
                        problem_id,
                    }
                } else if texts
                    .iter()
                    .flat_map(|s| s.status.clone())
                    .any(|s| s.to_lowercase() == "pending")
                {
                    BadgeTemplate {
                        text: "チェック".to_string(),
                        color: "text-bg-warning".to_string(),
                        should_refresh: true,
                        problem_id,
                    }
                } else if texts
                    .iter()
                    .flat_map(|s| s.status.clone())
                    .any(|s| s.to_lowercase() == "wa")
                {
                    BadgeTemplate {
                        text: "失敗".to_string(),
                        color: "text-bg-danger".to_string(),
                        should_refresh: false,
                        problem_id,
                    }
                } else {
                    BadgeTemplate {
                        text: "不明".to_string(),
                        color: "badge text-bg-secondary".to_string(),
                        should_refresh: false,
                        problem_id,
                    }
                }
                .into_response()
            }

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn solution_badge(
        auth_session: AuthSession,
        State(state): State<ServerState>,
        Path(solution_id): Path<String>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(_) => {
                let texts = sqlx::query!("select status from solutions where id = ?", solution_id)
                    .fetch_one(&state.db)
                    .await
                    .unwrap();
                match texts.status {
                    Some(status) => match status.to_lowercase().as_str() {
                        "ac" => SolutionBadgeTemplate {
                            text: "完了".to_string(),
                            color: "text-bg-success".to_string(),
                            should_refresh: false,
                            solution_id,
                        },
                        "wa" => SolutionBadgeTemplate {
                            text: "失敗".to_string(),
                            color: "text-bg-danger".to_string(),
                            should_refresh: false,
                            solution_id,
                        },
                        "pending" => SolutionBadgeTemplate {
                            text: "チェック".to_string(),
                            color: "text-bg-warning".to_string(),
                            should_refresh: true,
                            solution_id,
                        },
                        _ => SolutionBadgeTemplate {
                            text: "不明".to_string(),
                            color: "badge text-bg-secondary".to_string(),
                            should_refresh: false,
                            solution_id,
                        },
                    },
                    None => SolutionBadgeTemplate {
                        text: "未完了".to_string(),
                        color: "text-bg-light".to_string(),
                        should_refresh: false,
                        solution_id,
                    },
                }
                .into_response()
            }

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    // pub async fn solution_output(
    //     auth_session: AuthSession,
    //     State(state): State<ServerState>,
    //     Path(solution_id): Path<String>,
    // ) -> impl IntoResponse {
    //     match auth_session.user {
    //         Some(user) =>

    //         None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    //     }
    // }

    pub async fn status(
        auth_session: AuthSession,
        messages: Messages,
        State(state): State<ServerState>,
        Path(problem_id): Path<String>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let record = sqlx::query!(
                    "select id,stdout,stderr from solutions where userid = ? and problem_id = ?",
                    user.id,
                    problem_id
                )
                .fetch_all(&state.db)
                .await
                .unwrap();
                StatusTemplate {
                    messages: messages.into_iter().collect(),
                    username: user.username,
                    solutions: record
                        .iter()
                        .map(|r| Solution {
                            id: r.id,
                            stdout: r.stdout.clone(),
                            stderr: r.stderr.clone(),
                            should_refresh: r.stdout.is_none(),
                        })
                        .collect(),
                    problem_id,
                }
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

mod post {
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::Redirect,
        Form,
    };
    use serde::Deserialize;

    use crate::{config::Task, users::AuthSession};

    use super::ServerState;

    #[derive(Debug, Clone, Deserialize)]
    pub struct SolvePost {
        answer: String,
    }

    pub async fn solve(
        auth_session: AuthSession,
        Path(id): Path<String>,
        State(state): State<ServerState>,
        Form(form): Form<SolvePost>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let output = sqlx::query!(
                    "insert into solutions (content,status,userid,problem_id) values (?,?,?,?) returning id",
                    form.answer,
                    "Pending",
                    user.id,
                    id
                )
                .fetch_one(&state.db)
                .await
                .unwrap();
                state.tx.send(Task { id: output.id }).await.unwrap();
                Redirect::to("/problems").into_response()
            }

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
