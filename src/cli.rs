use clap::{Parser, Subcommand};
use reqwest::Client;
use crate::config::{get_adjent_home, detect_context, Context};
use crate::storage::LocalStorage;
use crate::server::{
    Project, ProjectCreate,
    Task, TaskCreate,
    Round, RoundCreate,
    ActionRequest, Action, ActionStatus, ActionStatusUpdate
};
use anyhow::Result;
use tracing::info;

#[derive(Parser)]
#[command(name = "adjent", about = "Adjent: An orchestrator for agents with human in the loop")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Project(ProjectArgs),
    Task(TaskArgs),
    Round(RoundArgs),
    Server(ServerArgs),
    Manage(ManageArgs),
}

#[derive(Parser)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub command: ProjectCommand,
}

#[derive(Subcommand)]
pub enum ProjectCommand {
    List,
    Create { id: String },
    Activate { id: String },
}

#[derive(Parser)]
pub struct TaskArgs {
    #[arg(short, long)]
    pub project_id: Option<String>,
    #[command(subcommand)]
    pub command: TaskCommand,
}

#[derive(Subcommand)]
pub enum TaskCommand {
    List,
    Create { id: String },
    Activate { id: String },
}

#[derive(Parser)]
pub struct RoundArgs {
    #[arg(short, long)]
    pub project_id: Option<String>,
    #[arg(short, long)]
    pub task_id: Option<String>,
    #[command(subcommand)]
    pub command: RoundCommand,
}

#[derive(Subcommand)]
pub enum RoundCommand {
    Activate { id: String },
    Bump {
        #[arg(long)]
        from: Option<String>
    },
    Add {
        #[command(subcommand)]
        item: AddItem,
    },
    Do { action: String },
}

#[derive(Subcommand)]
pub enum AddItem {
    Input { artifact: String },
}

#[derive(Parser)]
pub struct ServerArgs {
    #[command(subcommand)]
    pub command: ServerCommand,
}

#[derive(Subcommand)]
pub enum ServerCommand {
    Start {
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    Stop,
}

#[derive(Parser)]
pub struct ManageArgs {
    #[arg(short, long)]
    pub project: Option<String>,
    #[arg(short, long)]
    pub agent: String,
}

pub struct CliContext {
    pub client: Client,
    pub storage: LocalStorage,
    pub home: std::path::PathBuf,
}

impl CliContext {
    pub fn new() -> Self {
        let home = get_adjent_home();
        Self {
            client: Client::new(),
            storage: LocalStorage::new(home.clone()),
            home,
        }
    }

    pub fn resolve_context(&self, explicit_p: Option<String>, explicit_t: Option<String>, explicit_r: Option<String>) -> Result<Context> {
        let cwd = std::env::current_dir()?;
        let cwd_ctx = detect_context(&cwd, &self.home);
        let active_ctx = self.storage.get_active_context().unwrap_or_default();

        Ok(Context {
            project: explicit_p.or(cwd_ctx.project).or(active_ctx.project),
            task: explicit_t.or(cwd_ctx.task).or(active_ctx.task),
            round: explicit_r.or(cwd_ctx.round).or(active_ctx.round),
        })
    }
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let ctx = CliContext::new();
    let base_url = "http://localhost:8080"; // Should be configurable

    match cli.command {
        Command::Project(args) => match args.command {
            ProjectCommand::List => {
                let res = ctx.client.get(format!("{}/projects", base_url)).send().await?;
                let projects: Vec<Project> = res.json().await?;
                let active_ctx = ctx.storage.get_active_context().unwrap_or_default();
                let cwd = std::env::current_dir()?;
                let cwd_ctx = detect_context(&cwd, &ctx.home);

                println!("Projects:");
                for p in projects {
                    let is_active = Some(&p.id) == active_ctx.project.as_ref() || Some(&p.id) == cwd_ctx.project.as_ref();
                    println!("{} {}", if is_active { "*" } else { "-" }, p.id);
                }
            },
            ProjectCommand::Create { id } => {
                let res = ctx.client.post(format!("{}/projects", base_url))
                    .json(&ProjectCreate { id: id.clone() })
                    .send().await?;
                if res.status().is_success() {
                    println!("Project created: {}", id);
                } else {
                    eprintln!("Failed to create project: {}", res.status());
                }
            },
            ProjectCommand::Activate { id } => {
                // Verify project exists (optional but good)
                let mut current = ctx.storage.get_active_context().unwrap_or_default();
                current.project = Some(id.clone());
                ctx.storage.save_active_context(&current)?;
                println!("Project activated: {}", id);
            },
        },
        Command::Task(args) => {
            let context = ctx.resolve_context(args.project_id, None, None)?;
            let p_id = context.project.expect("Project ID is required");
            match args.command {
                TaskCommand::List => {
                    let res = ctx.client.get(format!("{}/projects/{}/tasks", base_url, p_id)).send().await?;
                    let tasks: Vec<Task> = res.json().await?;
                    let active_ctx = ctx.storage.get_active_context().unwrap_or_default();
                    let cwd = std::env::current_dir()?;
                    let cwd_ctx = detect_context(&cwd, &ctx.home);

                    println!("Tasks for project {}:", p_id);
                    for t in tasks {
                        let is_active = Some(&t.id) == active_ctx.task.as_ref() || Some(&t.id) == cwd_ctx.task.as_ref();
                        println!("{} {}", if is_active { "*" } else { "-" }, t.id);
                    }
                },
                TaskCommand::Create { id } => {
                    let res = ctx.client.post(format!("{}/projects/{}/tasks", base_url, p_id))
                        .json(&TaskCreate { name: id.clone() })
                        .send().await?;
                    if res.status().is_success() {
                        let task: Task = res.json().await?;
                        println!("Task created: {}", task.id);
                    } else {
                        eprintln!("Failed to create task: {}", res.status());
                    }
                },
                TaskCommand::Activate { id } => {
                    let mut current = ctx.storage.get_active_context().unwrap_or_default();
                    current.project = Some(p_id);
                    current.task = Some(id.clone());
                    ctx.storage.save_active_context(&current)?;
                    println!("Task activated: {}", id);
                },
            }
        },
        Command::Round(args) => {
            let context = ctx.resolve_context(args.project_id, args.task_id, None)?;
            let p_id = context.project.expect("Project ID is required");
            let t_id = context.task.expect("Task ID is required");
            match args.command {
                RoundCommand::Activate { id } => {
                    let mut current = ctx.storage.get_active_context().unwrap_or_default();
                    current.project = Some(p_id);
                    current.task = Some(t_id);
                    current.round = Some(id.clone());
                    ctx.storage.save_active_context(&current)?;
                    println!("Round activated: {}", id);
                },
                RoundCommand::Bump { from } => {
                    let res = ctx.client.post(format!("{}/projects/{}/tasks/{}/rounds", base_url, p_id, t_id))
                        .json(&RoundCreate { from_round_id: from })
                        .send().await?;
                    if res.status().is_success() {
                        let round: Round = res.json().await?;
                        println!("Round bumped: {}", round.id);
                    } else {
                        eprintln!("Failed to bump round: {}", res.status());
                    }
                },
                RoundCommand::Add { item } => match item {
                    AddItem::Input { artifact } => println!("Adding input: {} to round in project: {}, task: {}", artifact, p_id, t_id),
                },
                RoundCommand::Do { action } => {
                    let r_id = context.round.expect("Round ID is required");
                    let res = ctx.client.post(format!("{}/projects/{}/tasks/{}/rounds/{}/do", base_url, p_id, t_id, r_id))
                        .json(&ActionRequest { action: action.clone() })
                        .send().await?;
                    if res.status().is_success() {
                        println!("Action {} accepted", action);
                    } else {
                        eprintln!("Failed to do action: {}", res.status());
                    }
                },
            }
        },
        Command::Server(args) => match args.command {
            ServerCommand::Start { port } => {
                println!("Starting server on port: {}", port);
                crate::server::start(port).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
            },
            ServerCommand::Stop => println!("Stopping server..."),
        },
        Command::Manage(args) => {
            run_manager(ctx, args, base_url).await?;
        }
    }

    Ok(())
}

async fn run_manager(ctx: CliContext, args: ManageArgs, base_url: &str) -> Result<()> {
    let project_id = ctx.resolve_context(args.project, None, None)?.project
        .expect("Project ID is required");
    
    info!("Starting manager for project: {} with agent: {}", project_id, args.agent);

    loop {
        // Poll for next action
        let res = ctx.client.get(format!("{}/projects/{}/actions/next", base_url, project_id))
            .send().await?;
        
        if res.status().is_success() {
            let action: Option<Action> = res.json().await?;
            
            if let Some(action) = action {
                info!("Assigned action: {} ({})", action.id, action.action);
                
                // Update status to running
                let _ = ctx.client.post(format!("{}/projects/{}/actions/{}/status", base_url, project_id, action.id))
                    .json(&ActionStatusUpdate { status: ActionStatus::Running })
                    .send().await?;

                // Spawn agent
                let success = spawn_agent(&action, &args.agent).await?;

                // Update status to completed/failed
                let status = if success { ActionStatus::Completed } else { ActionStatus::Failed };
                let _ = ctx.client.post(format!("{}/projects/{}/actions/{}/status", base_url, project_id, action.id))
                    .json(&ActionStatusUpdate { status })
                    .send().await?;
                
                info!("Action {} {}", action.id, if success { "completed" } else { "failed" });
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn spawn_agent(action: &Action, agent_command: &str) -> Result<bool> {
    let full_command = format!("{} /{}", agent_command, action.action);
    let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(full_command)
        .env("ADJENT_PROJECT_ID", &action.project_id)
        .env("ADJENT_TASK_ID", &action.task_id)
        .env("ADJENT_ROUND_ID", &action.round_id)
        .env("ADJENT_ACTION_ID", &action.id)
        .env("ADJENT_ACTION", &action.action)
        .spawn()?;

    let status = child.wait().await?;
    Ok(status.success())
}
