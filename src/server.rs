use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use tracing::instrument;

use crate::Task;

#[instrument(skip_all)]
pub async fn server_main(task_sender: Sender<Task>) -> anyhow::Result<()> {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .with_state(task_sender);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("listen on port")?;
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
