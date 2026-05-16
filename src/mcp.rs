use crate::storage::{LocalStorage, ArtifactType};
use serde::Deserialize;
use schemars::JsonSchema;
use rmcp::{tool, tool_router, ErrorData, RoleServer};
use rmcp::model::{CallToolResult, Content};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::service::RequestContext;
use std::sync::Arc;

pub struct McpSessionContext {
    pub project_id: String,
    pub task_id: String,
    pub round_id: String,
}

pub struct AdjentMcpServer {
    pub storage: Arc<LocalStorage>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListArtifactsParams {
    /// The directory type: "inputs", "outputs", or "logs"
    #[serde(rename = "type")]
    pub artifact_type: ArtifactType,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReadArtifactParams {
    /// The directory type: "inputs", "outputs", or "logs"
    #[serde(rename = "type")]
    pub artifact_type: ArtifactType,
    /// The name of the file to read
    pub filename: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct WriteArtifactParams {
    /// The directory type: "inputs", "outputs", or "logs"
    #[serde(rename = "type")]
    pub artifact_type: ArtifactType,
    /// The name of the file to write
    pub filename: String,
    /// The content to write to the file
    pub content: String,
}

impl AdjentMcpServer {
    fn resolve_context(&self, context: &RequestContext<RoleServer>) -> Result<McpSessionContext, ErrorData> {
        let parts = context.extensions.get::<axum::http::request::Parts>()
            .ok_or_else(|| ErrorData::internal_error("Missing request parts in extensions", None))?;
        
        let project_id = parts.headers.get("X-Adjent-ProjectId")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| ErrorData::invalid_params("Missing X-Adjent-ProjectId header", None))?;
            
        let action_id = parts.headers.get("X-Adjent-ActionId")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| ErrorData::invalid_params("Missing X-Adjent-ActionId header", None))?;

        let action = self.storage.get_action(project_id, action_id)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?
            .ok_or_else(|| ErrorData::invalid_params("Action not found", None))?;

        Ok(McpSessionContext {
            project_id: action.project_id,
            task_id: action.task_id,
            round_id: action.round_id,
        })
    }
}

#[tool_router]
impl AdjentMcpServer {
    #[tool(description = "List files in a specific artifact directory.")]
    pub async fn list_artifacts(&self, context: RequestContext<RoleServer>, Parameters(params): Parameters<ListArtifactsParams>) -> Result<CallToolResult, ErrorData> {
        let ctx = self.resolve_context(&context)?;
        
        let files = self.storage.list_artifacts(
            &ctx.project_id,
            &ctx.task_id,
            &ctx.round_id,
            params.artifact_type
        ).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        
        Ok(CallToolResult::success(vec![Content::text(files.join("\n"))]))
    }

    #[tool(description = "Read the content of an artifact file.")]
    pub async fn read_artifact(&self, context: RequestContext<RoleServer>, Parameters(params): Parameters<ReadArtifactParams>) -> Result<CallToolResult, ErrorData> {
        let ctx = self.resolve_context(&context)?;
        
        let path = self.storage.get_artifact_path(
            &ctx.project_id,
            &ctx.task_id,
            &ctx.round_id,
            params.artifact_type,
            &params.filename
        );

        if !path.exists() {
            return Err(ErrorData::invalid_params(format!("File not found: {}", params.filename), None));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        
        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(description = "Write or update an artifact file.")]
    pub async fn write_artifact(&self, context: RequestContext<RoleServer>, Parameters(params): Parameters<WriteArtifactParams>) -> Result<CallToolResult, ErrorData> {
        let ctx = self.resolve_context(&context)?;
        
        let path = self.storage.get_artifact_path(
            &ctx.project_id,
            &ctx.task_id,
            &ctx.round_id,
            params.artifact_type,
            &params.filename
        );

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        }

        std::fs::write(path, params.content)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        
        Ok(CallToolResult::success(vec![Content::text(format!("Successfully wrote {}", params.filename))]))
    }
}

impl rmcp::ServerHandler for AdjentMcpServer {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{LocalStorage, Action, ActionStatus};
    use tempfile::tempdir;
    use rmcp::model::{NumberOrString, ClientJsonRpcMessage, ClientNotification, RawContent, JsonRpcVersion2_0};
    use axum::http::Request;
    use rmcp::transport::OneshotTransport;
    use rmcp::service::serve_directly;

    #[tokio::test]
    async fn test_tool_logic() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let storage = Arc::new(LocalStorage::new(dir.path().to_path_buf()));
        let server = AdjentMcpServer { storage: storage.clone() };

        // Setup a project, task, and action
        storage.create_project("p1")?;
        storage.create_task("p1", "t1")?; // creates round 0
        let action = Action {
            id: "a1".into(),
            project_id: "p1".into(),
            task_id: "t1".into(),
            round_id: "0".into(),
            action: "test".into(),
            status: ActionStatus::Pending,
            created_at: chrono::Utc::now(),
        };
        storage.save_action("p1", &action)?;

        // Obtain a Peer by serving a dummy OneshotTransport
        let dummy_msg = ClientJsonRpcMessage::Notification(rmcp::model::JsonRpcNotification {
            jsonrpc: JsonRpcVersion2_0::default(),
            notification: ClientNotification::InitializedNotification(Default::default()),
        });
        let (transport, _) = OneshotTransport::<RoleServer>::new(dummy_msg);
        let running = serve_directly(AdjentMcpServer { storage: storage.clone() }, transport, None);
        let peer = running.peer().clone();
        
        let mut context = RequestContext::new(NumberOrString::Number(1), peer);
        
        let req = Request::builder()
            .header("X-Adjent-ProjectId", "p1")
            .header("X-Adjent-ActionId", "a1")
            .body(())?;
        let (parts, _) = req.into_parts();
        context.extensions.insert(parts);

        // Test write_artifact
        let write_params = WriteArtifactParams {
            artifact_type: ArtifactType::Inputs,
            filename: "test.txt".into(),
            content: "hello world".into(),
        };
        server.write_artifact(context.clone(), Parameters(write_params)).await.expect("write failed");

        // Test list_artifacts
        let list_params = ListArtifactsParams {
            artifact_type: ArtifactType::Inputs,
        };
        let res = server.list_artifacts(context.clone(), Parameters(list_params)).await.expect("list failed");
        let text = match &res.content[0].raw {
            RawContent::Text(t) => &t.text,
            _ => panic!("Expected text content"),
        };
        assert!(text.contains("test.txt"));

        // Test read_artifact
        let read_params = ReadArtifactParams {
            artifact_type: ArtifactType::Inputs,
            filename: "test.txt".into(),
        };
        let res = server.read_artifact(context.clone(), Parameters(read_params)).await.expect("read failed");
        let text = match &res.content[0].raw {
            RawContent::Text(t) => &t.text,
            _ => panic!("Expected text content"),
        };
        assert_eq!(text, "hello world");

        Ok(())
    }
}
