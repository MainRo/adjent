use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use crate::config::Context;

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
}
