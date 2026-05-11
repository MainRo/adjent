use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use crate::config::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActionStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    pub id: String,
    pub project_id: String,
    pub task_id: String,
    pub round_id: String,
    pub action: String,
    pub status: ActionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct LocalStorage {
    root: PathBuf,
}

impl LocalStorage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn projects_dir(&self) -> PathBuf {
        self.root.join("state").join("projects")
    }

    fn tasks_dir(&self, project_id: &str) -> PathBuf {
        self.projects_dir().join(project_id).join("tasks")
    }

    fn actions_dir(&self, project_id: &str) -> PathBuf {
        self.projects_dir().join(project_id).join("actions")
    }

    fn rounds_dir(&self, project_id: &str, task_id: &str) -> PathBuf {
        self.tasks_dir(project_id).join(task_id).join("rounds")
    }

    pub fn save_action(&self, project_id: &str, action: &Action) -> Result<()> {
        let dir = self.actions_dir(project_id);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", action.id));
        let content = serde_json::to_string_pretty(action)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_action(&self, project_id: &str, action_id: &str) -> Result<Option<Action>> {
        let path = self.actions_dir(project_id).join(format!("{}.json", action_id));
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let action = serde_json::from_str(&content)?;
        Ok(Some(action))
    }

    pub fn update_action_status(&self, project_id: &str, action_id: &str, status: ActionStatus) -> Result<()> {
        let mut action = self.get_action(project_id, action_id)?
            .ok_or_else(|| anyhow::anyhow!("Action not found"))?;
        action.status = status;
        self.save_action(project_id, &action)?;
        Ok(())
    }

    pub fn get_and_assign_next_action(&self, project_id: &str) -> Result<Option<Action>> {
        let dir = self.actions_dir(project_id);
        if !dir.exists() {
            return Ok(None);
        }

        let mut entries: Vec<_> = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .collect();
        
        // Sort by creation time (optional, but good for FIFO)
        entries.sort_by_key(|e| e.metadata().and_then(|m| m.created()).ok());

        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let mut action: Action = serde_json::from_str(&content)?;
                
                if action.status == ActionStatus::Pending {
                    action.status = ActionStatus::Assigned;
                    let updated_content = serde_json::to_string_pretty(&action)?;
                    fs::write(&path, updated_content)?;
                    return Ok(Some(action));
                }
            }
        }

        Ok(None)
    }

    pub fn list_rounds(&self, project_id: &str, task_id: &str) -> Result<Vec<String>> {
        let path = self.rounds_dir(project_id, task_id);
        if !path.exists() {
            return Ok(vec![]);
        }
        let mut rounds = vec![];
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    rounds.push(name.to_string());
                }
            }
        }
        Ok(rounds)
    }

    pub fn create_round_dir(&self, project_id: &str, task_id: &str, round_id: &str) -> Result<()> {
        let path = self.rounds_dir(project_id, task_id).join(round_id);
        fs::create_dir_all(path.join("inputs"))?;
        fs::create_dir_all(path.join("outputs"))?;
        fs::create_dir_all(path.join("logs"))?;
        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<String>> {
        let path = self.projects_dir();
        if !path.exists() {
            return Ok(vec![]);
        }

        let mut projects = vec![];
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    projects.push(name.to_string());
                }
            }
        }
        Ok(projects)
    }

    pub fn create_project(&self, id: &str) -> Result<()> {
        let path = self.projects_dir().join(id);
        fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn list_tasks(&self, project_id: &str) -> Result<Vec<String>> {
        let path = self.tasks_dir(project_id);
        if !path.exists() {
            return Ok(vec![]);
        }
        let mut tasks = vec![];
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    tasks.push(name.to_string());
                }
            }
        }
        Ok(tasks)
    }

    pub fn create_task(&self, project_id: &str, task_id: &str) -> Result<()> {
        let path = self.tasks_dir(project_id).join(task_id);
        fs::create_dir_all(path)?;
        self.bump_round(project_id, task_id, None)?;
        Ok(())
    }

    pub fn bump_round(&self, project_id: &str, task_id: &str, from_r_id: Option<String>) -> Result<String> {
        let rounds = self.list_rounds(project_id, task_id)?;
        let src_id = from_r_id.or_else(|| {
            rounds.iter().filter_map(|s| s.parse::<u32>().ok()).max().map(|m| m.to_string())
        });

        let next_id = src_id.as_ref()
            .and_then(|id| id.parse::<u32>().ok())
            .map(|n| n + 1)
            .unwrap_or(0)
            .to_string();

        let target_dir = self.rounds_dir(project_id, task_id).join(&next_id);
        self.create_round_dir(project_id, task_id, &next_id)?;

        if let Some(sid) = src_id {
            let src_dir = self.rounds_dir(project_id, task_id).join(sid);
            self.copy_artifacts(&src_dir.join("inputs"), &target_dir.join("inputs"))?;
            self.copy_artifacts(&src_dir.join("outputs"), &target_dir.join("inputs"))?;
        }
        
        fs::write(target_dir.join("inputs").join("instructions.md"), "")?;
        Ok(next_id)
    }

    fn copy_artifacts(&self, src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
        if !src.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let name = entry.file_name();
                fs::copy(&path, dst.join(name))?;
            }
        }
        Ok(())
    }

    pub fn get_active_context(&self) -> Result<Context> {
        let path = self.root.join("active.json");
        if !path.exists() {
            return Ok(Context::default());
        }
        let content = fs::read_to_string(path)?;
        let ctx = serde_json::from_str(&content)?;
        Ok(ctx)
    }

    pub fn save_active_context(&self, ctx: &Context) -> Result<()> {
        let path = self.root.join("active.json");
        let content = serde_json::to_string_pretty(ctx)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_list_projects_empty() -> Result<()> {
        let dir = tempdir()?;
        let storage = LocalStorage::new(dir.path().to_path_buf());
        assert!(storage.list_projects()?.is_empty());
        Ok(())
    }

    #[test]
    fn test_create_and_list_project() -> Result<()> {
        let dir = tempdir()?;
        let storage = LocalStorage::new(dir.path().to_path_buf());
        storage.create_project("p1")?;
        let projects = storage.list_projects()?;
        assert_eq!(projects, vec!["p1".to_string()]);
        Ok(())
    }

    #[test]
    fn test_active_context() -> Result<()> {
        let dir = tempdir()?;
        let storage = LocalStorage::new(dir.path().to_path_buf());
        let ctx = Context {
            project: Some("p1".into()),
            task: Some("t1".into()),
            round: None,
        };
        storage.save_active_context(&ctx)?;
        let loaded = storage.get_active_context()?;
        assert_eq!(loaded, ctx);
        Ok(())
    }

    #[test]
    fn test_create_and_list_tasks() -> Result<()> {
        let dir = tempdir()?;
        let storage = LocalStorage::new(dir.path().to_path_buf());
        storage.create_project("p1")?;
        storage.create_task("p1", "t1")?;
        let tasks = storage.list_tasks("p1")?;
        assert_eq!(tasks, vec!["t1".to_string()]);
        Ok(())
    }

    #[test]
    fn test_bump_round() -> Result<()> {
        let dir = tempdir()?;
        let storage = LocalStorage::new(dir.path().to_path_buf());
        storage.create_project("p1")?;
        storage.create_task("p1", "t1")?; // creates round 0
        
        let rounds = storage.list_rounds("p1", "t1")?;
        assert_eq!(rounds, vec!["0".to_string()]);

        // Add some "artifacts" to round 0
        let r0_dir = storage.rounds_dir("p1", "t1").join("0");
        fs::write(r0_dir.join("inputs").join("data.txt"), "hello")?;
        fs::write(r0_dir.join("outputs").join("result.txt"), "world")?;

        let next = storage.bump_round("p1", "t1", None)?;
        assert_eq!(next, "1");

        let r1_dir = storage.rounds_dir("p1", "t1").join("1");
        assert!(r1_dir.join("inputs").join("data.txt").exists());
        assert!(r1_dir.join("inputs").join("result.txt").exists());
        assert!(r1_dir.join("inputs").join("instructions.md").exists());
        
        assert_eq!(fs::read_to_string(r1_dir.join("inputs").join("data.txt"))?, "hello");
        assert_eq!(fs::read_to_string(r1_dir.join("inputs").join("result.txt"))?, "world");

        Ok(())
    }
}
