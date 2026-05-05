use axum::{
    extract::{Path, Multipart},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectCreate {
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct TaskCreate {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Round {
    pub id: String,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct RoundCreate {
    pub from_round_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ActionRequest {
    pub action: String,
}

pub async fn start(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/:projectId", get(get_project))
        .route("/projects/:projectId/tasks", get(list_tasks).post(create_task))
        .route("/projects/:projectId/tasks/:taskId", get(get_task))
        .route("/projects/:projectId/tasks/:taskId/rounds", get(list_rounds).post(create_round))
        .route("/projects/:projectId/tasks/:taskId/rounds/:roundId/artifacts/:type", get(list_artifacts).post(upload_artifact))
        .route("/projects/:projectId/tasks/:taskId/rounds/:roundId/artifacts/:type/:filename", get(download_artifact))
        .route("/projects/:projectId/tasks/:taskId/rounds/:roundId/do", post(do_action))
        .route("/mcp", get(crate::mcp::mcp_sse_handler).post(crate::mcp::mcp_post_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_projects() -> Json<Vec<Project>> {
    Json(vec![Project { id: "myProject".into() }])
}

async fn create_project(Json(payload): Json<ProjectCreate>) -> (axum::http::StatusCode, Json<Project>) {
    (axum::http::StatusCode::CREATED, Json(Project { id: payload.id }))
}

async fn get_project(Path(project_id): Path<String>) -> Json<Project> {
    Json(Project { id: project_id })
}

async fn list_tasks(Path(_project_id): Path<String>) -> Json<Vec<Task>> {
    Json(vec![Task { id: "20260302-foo".into(), name: "foo".into() }])
}

async fn create_task(Path(_project_id): Path<String>, Json(payload): Json<TaskCreate>) -> (axum::http::StatusCode, Json<Task>) {
    (axum::http::StatusCode::CREATED, Json(Task { id: format!("20260302-{}", payload.name), name: payload.name }))
}

async fn get_task(Path((_project_id, task_id)): Path<(String, String)>) -> Json<Task> {
    Json(Task { id: task_id.clone(), name: task_id })
}

async fn list_rounds(Path((_project_id, _task_id)): Path<(String, String)>) -> Json<Vec<Round>> {
    Json(vec![Round { id: "round-1".into(), status: "completed".into() }])
}

async fn create_round(Path((_project_id, _task_id)): Path<(String, String)>, Json(_payload): Json<RoundCreate>) -> (axum::http::StatusCode, Json<Round>) {
    (axum::http::StatusCode::CREATED, Json(Round { id: "round-2".into(), status: "pending".into() }))
}

async fn list_artifacts(Path((_project_id, _task_id, _round_id, _artifact_type)): Path<(String, String, String, String)>) -> Json<Vec<String>> {
    Json(vec!["plan.md".into()])
}

async fn upload_artifact(Path((_project_id, _task_id, _round_id, _artifact_type)): Path<(String, String, String, String)>, mut _multipart: Multipart) -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}

async fn download_artifact(Path((_project_id, _task_id, _round_id, _artifact_type, _filename)): Path<(String, String, String, String, String)>) -> &'static str {
    "artifact content"
}

async fn do_action(Path((_project_id, _task_id, _round_id)): Path<(String, String, String)>, Json(_payload): Json<ActionRequest>) -> axum::http::StatusCode {
    axum::http::StatusCode::ACCEPTED
}
