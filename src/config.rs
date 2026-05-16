use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Context {
    pub project: Option<String>,
    pub task: Option<String>,
    pub round: Option<String>,
}

pub fn get_adjent_home() -> PathBuf {
    let path = std::env::var("ADJENT_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME env var not set");
            PathBuf::from(home).join(".adjent")
        });
    path.canonicalize().unwrap_or(path)
}

pub fn detect_context(cwd: &Path, adjent_home: &Path) -> Context {
    let cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let adjent_home = adjent_home.canonicalize().unwrap_or_else(|_| adjent_home.to_path_buf());

    if let Ok(rel) = cwd.strip_prefix(adjent_home) {
        let components: Vec<_> = rel.components()
            .map(|c| c.as_os_str().to_str().unwrap_or(""))
            .collect();

        // Expected structure: state/projects/[project]/tasks/[task]/rounds/[round]
        if components.len() >= 3 && components[0] == "state" && components[1] == "projects" {
            let project = Some(components[2].to_string());
            let task = if components.len() >= 5 && components[3] == "tasks" {
                Some(components[4].to_string())
            } else {
                None
            };
            let round = if components.len() >= 7 && components[5] == "rounds" {
                Some(components[6].to_string())
            } else {
                None
            };
            return Context { project, task, round };
        }
    }
    Context::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_context_inside_project() {
        let home = PathBuf::from("/home/user");
        let cwd = PathBuf::from("/home/user/state/projects/proj1/tasks/task1/rounds/0");
        let ctx = detect_context(&cwd, &home);
        assert_eq!(ctx.project, Some("proj1".to_string()));
        assert_eq!(ctx.task, Some("task1".to_string()));
        assert_eq!(ctx.round, Some("0".to_string()));
    }

    #[test]
    fn test_detect_context_partial() {
        let home = PathBuf::from("/home/user");
        let cwd = PathBuf::from("/home/user/state/projects/proj1");
        let ctx = detect_context(&cwd, &home);
        assert_eq!(ctx.project, Some("proj1".to_string()));
        assert_eq!(ctx.task, None);
        assert_eq!(ctx.round, None);
    }

    #[test]
    fn test_detect_context_outside() {
        let home = PathBuf::from("/home/user");
        let cwd = PathBuf::from("/home/other");
        let ctx = detect_context(&cwd, &home);
        assert_eq!(ctx.project, None);
    }

    #[test]
    fn test_get_adjent_home_env() {
        // Grouping environment-modifying tests to avoid parallel race conditions
        unsafe {
            let original_adjent_home = std::env::var("ADJENT_HOME");
            let original_home = std::env::var("HOME");

            // Test with ADJENT_HOME set
            std::env::set_var("ADJENT_HOME", "/tmp");
            assert!(get_adjent_home().to_str().unwrap().contains("tmp"));

            // Test default behavior (fallback to HOME)
            std::env::remove_var("ADJENT_HOME");
            std::env::set_var("HOME", "/home/user");
            assert_eq!(get_adjent_home(), PathBuf::from("/home/user/.adjent"));

            // Restore original environment
            if let Ok(val) = original_adjent_home {
                std::env::set_var("ADJENT_HOME", val);
            } else {
                std::env::remove_var("ADJENT_HOME");
            }
            if let Ok(val) = original_home {
                std::env::set_var("HOME", val);
            }
        }
    }
}
