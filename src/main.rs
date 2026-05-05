mod cli;
mod server;
mod mcp;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(e) = cli::run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
