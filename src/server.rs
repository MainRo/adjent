use axum::{
    extract::{Path, Multipart, State},
    routing::{get, post},
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, error};
pub use crate::storage::{LocalStorage, Action, ActionStatus};
use crate::config::get_adjent_home;
use chrono::Local;
use crate::mcp::AdjentMcpServer;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

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

#[derive(Serialize, Deserialize)]
pub struct ActionStatusUpdate {
    pub status: ActionStatus,
}

pub struct AppState {
    pub storage: Arc<LocalStorage>,
    pub mcp_service: StreamableHttpService<AdjentMcpServer, LocalSessionManager>,
}

pub async fn start(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let home = get_adjent_home();
    let storage = Arc::new(LocalStorage::new(home));
    let session_manager = Arc::new(LocalSessionManager::default());
    
    let storage_clone = storage.clone();
    let mcp_service = StreamableHttpService::new(
        move || Ok(AdjentMcpServer { storage: storage_clone.clone() }),
        session_manager,
        StreamableHttpServerConfig::default(),
    );

    let state = Arc::new(AppState { storage, mcp_service });

    let app = Router::new()
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/:projectId", get(get_project))
        .route("/projects/:projectId/tasks", get(list_tasks).post(create_task))
        .route("/projects/:projectId/tasks/:taskId", get(get_task))
        .route("/projects/:projectId/tasks/:taskId/rounds", get(list_rounds).post(create_round))
        .route("/projects/:projectId/tasks/:taskId/rounds/:roundId/artifacts/:type", get(list_artifacts).post(upload_artifact))
        .route("/projects/:projectId/tasks/:taskId/rounds/:roundId/artifacts/:type/:filename", get(download_artifact))
        .route("/projects/:projectId/tasks/:taskId/rounds/:roundId/do", post(do_action))
        .route("/projects/:projectId/actions/next", get(get_next_action))
        .route("/projects/:projectId/actions/:actionId/status", post(update_action_status))
        .route("/mcp", get(mcp_handler).post(mcp_handler).delete(mcp_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn mcp_handler(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    state.mcp_service.handle(req).await
}

async fn list_projects(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Project>>, StatusCode> {
    match state.storage.list_projects() {
        Ok(ids) => Ok(Json(ids.into_iter().map(|id| Project { id }).collect())),
        Err(e) => {
            error!("Failed to list projects: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProjectCreate>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    match state.storage.create_project(&payload.id) {
        Ok(_) => Ok((StatusCode::CREATED, Json(Project { id: payload.id }))),
        Err(e) => {
            error!("Failed to create project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project(
    State(_state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Json<Project> {
    Json(Project { id: project_id })
}

async fn list_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    match state.storage.list_tasks(&project_id) {
        Ok(ids) => Ok(Json(ids.into_iter().map(|id| {
            let name = id.splitn(2, '-').nth(1).unwrap_or(&id).to_string();
            Task { id, name }
        }).collect())),
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
    Json(payload): Json<TaskCreate>,
) -> Result<(StatusCode, Json<Task>), StatusCode> {
    let prefix = Local::now().format("%Y%m%d").to_string();
    let task_id = format!("{}-{}", prefix, payload.name);
    
    match state.storage.create_task(&project_id, &task_id) {
        Ok(_) => Ok((StatusCode::CREATED, Json(Task { id: task_id, name: payload.name }))),
        Err(e) => {
            error!("Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_task(
    State(_state): State<Arc<AppState>>,
    Path((_project_id, task_id)): Path<(String, String)>,
) -> Json<Task> {
    let name = task_id.splitn(2, '-').nth(1).unwrap_or(&task_id).to_string();
    Json(Task { id: task_id, name })
}

async fn list_rounds(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(String, String)>
) -> Result<Json<Vec<Round>>, StatusCode> {
    match state.storage.list_rounds(&project_id, &task_id) {
        Ok(ids) => Ok(Json(ids.into_iter().map(|id| Round { id, status: "active".into() }).collect())),
        Err(e) => {
            error!("Failed to list rounds: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_round(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(String, String)>,
    Json(payload): Json<RoundCreate>
) -> Result<(StatusCode, Json<Round>), StatusCode> {
    match state.storage.bump_round(&project_id, &task_id, payload.from_round_id) {
        Ok(id) => Ok((StatusCode::CREATED, Json(Round { id, status: "pending".into() }))),
        Err(e) => {
            error!("Failed to bump round: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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

async fn do_action(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id, round_id)): Path<(String, String, String)>,
    Json(payload): Json<ActionRequest>
) -> Result<StatusCode, StatusCode> {
    let action = Action {
        id: uuid::Uuid::new_v4().to_string(),
        project_id: project_id.clone(),
        task_id: task_id.clone(),
        round_id: round_id.clone(),
        action: payload.action,
        status: ActionStatus::Pending,
        created_at: chrono::Utc::now(),
    };

    match state.storage.save_action(&project_id, &action) {
        Ok(_) => Ok(StatusCode::ACCEPTED),
        Err(e) => {
            error!("Failed to save action: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_next_action(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<Option<Action>>, StatusCode> {
    match state.storage.get_and_assign_next_action(&project_id) {
        Ok(action) => Ok(Json(action)),
        Err(e) => {
            error!("Failed to get next action: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_action_status(
    State(state): State<Arc<AppState>>,
    Path((project_id, action_id)): Path<(String, String)>,
    Json(payload): Json<ActionStatusUpdate>,
) -> Result<StatusCode, StatusCode> {
    match state.storage.update_action_status(&project_id, &action_id, payload.status) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            error!("Failed to update action status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
