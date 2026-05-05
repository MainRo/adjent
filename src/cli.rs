use clap::{Parser, Subcommand};

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
    pub project_id: String,
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
    pub project_id: String,
    #[arg(short, long)]
    pub task_id: String,
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

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Project(args) => match args.command {
            ProjectCommand::List => println!("Listing projects..."),
            ProjectCommand::Create { id } => println!("Creating project: {}", id),
            ProjectCommand::Activate { id } => println!("Activating project: {}", id),
        },
        Command::Task(args) => match args.command {
            TaskCommand::List => println!("Listing tasks for project: {}", args.project_id),
            TaskCommand::Create { id } => println!("Creating task: {} in project: {}", id, args.project_id),
            TaskCommand::Activate { id } => println!("Activating task: {} in project: {}", id, args.project_id),
        },
        Command::Round(args) => match args.command {
            RoundCommand::Activate { id } => println!("Activating round: {} in project: {}, task: {}", id, args.project_id, args.task_id),
            RoundCommand::Bump { from } => println!("Bumping round from: {:?} in project: {}, task: {}", from, args.project_id, args.task_id),
            RoundCommand::Add { item } => match item {
                AddItem::Input { artifact } => println!("Adding input: {} to round in project: {}, task: {}", artifact, args.project_id, args.task_id),
            },
            RoundCommand::Do { action } => println!("Doing action: {} in project: {}, task: {}", action, args.project_id, args.task_id),
        },
        Command::Server(args) => match args.command {
            ServerCommand::Start { port } => {
                println!("Starting server on port: {}", port);
                crate::server::start(port).await?;
            },
            ServerCommand::Stop => println!("Stopping server..."),
        },
    }

    Ok(())
}
