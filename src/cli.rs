use clap::{Parser, Subcommand};
use reqwest::Client;
use crate::config::{get_adjent_home, detect_context, Context};
use crate::storage::LocalStorage;
use crate::server::{Project, ProjectCreate, Task, TaskCreate};
use anyhow::Result;

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
                println!("Projects:");
                for p in projects {
                    println!("- {}", p.id);
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
                    println!("Tasks for project {}:", p_id);
                    for t in tasks {
                        println!("- {}", t.id);
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
                RoundCommand::Bump { from } => println!("Bumping round from: {:?} in project: {}, task: {}", from, p_id, t_id),
                RoundCommand::Add { item } => match item {
                    AddItem::Input { artifact } => println!("Adding input: {} to round in project: {}, task: {}", artifact, p_id, t_id),
                },
                RoundCommand::Do { action } => println!("Doing action: {} in project: {}, task: {}", action, p_id, t_id),
            }
        },
        Command::Server(args) => match args.command {
            ServerCommand::Start { port } => {
                println!("Starting server on port: {}", port);
                crate::server::start(port).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
            },
            ServerCommand::Stop => println!("Stopping server..."),
        },
    }

    Ok(())
}
