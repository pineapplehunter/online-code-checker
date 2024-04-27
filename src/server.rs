use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tower_http::trace::{self, TraceLayer};
use tracing::{debug, Level};
use tracing_subscriber::field::debug;

use crate::Task;

pub async fn server_main(
    listener: TcpListener,
    task_sender: Sender<Task>,
    db: Pool<Sqlite>,
) -> anyhow::Result<()> {
    debug!("setup server");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .with_state(task_sender)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    axum::serve(listener, app).await.context("start server")?;

    Ok(())
}

// basic handler that responds with a static string
async fn root(State(sender): State<Sender<Task>>) -> &'static str {
    sender
        .send(Task {
            id: 0,
            filename: "".into(),
            inputs: "".into(),
        })
        .await
        .context("sending task to executor")
        .unwrap();
    "Hello, World!"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
