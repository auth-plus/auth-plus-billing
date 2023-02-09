pub mod commands;

use clap::{Parser, Subcommand};
use log::{error, info};

use self::commands::retry_charging_invoices::retry_charging_invoices;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// retry_charging_invoices
    Rci,
}

#[tokio::main]
pub async fn start() -> std::io::Result<()> {
    let cli = Cli::parse();
    info!("Job {:?} starting", cli.command);
    let result = match cli.command {
        Commands::Rci => retry_charging_invoices(),
    };

    if let Err(err) = result.await {
        error!("{:?}", err)
    }
    info!("Job {:?} finished", cli.command);
    Ok(())
}
