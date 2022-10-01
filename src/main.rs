use clap::{Parser, Subcommand};
use dotenv::dotenv;
use std::error::Error as StdError;
use summary::{run_summary, SummaryArgs};

mod summary;
mod time_utils;
mod toggl;

/// Utilities for working with the Toggl Track API
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Prints a summary of time tracking entries from a select time period
    #[command(verbatim_doc_comment)]
    Summary(SummaryArgs),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn StdError>> {
    dotenv().ok();
    let args = Cli::parse();

    match args.command {
        Command::Summary(args) => run_summary(args).await?,
    }

    Ok(())
}
