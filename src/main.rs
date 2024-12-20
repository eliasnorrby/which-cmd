mod config;
mod config_node;
mod constants;
mod options;
mod search;
mod terminal;
mod tui;

mod commands;

use commands::integration::Shell;

use clap::{command, Parser, Subcommand};
use std::error::Error;

/// A command builder tool – which-key for the command line
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Build a command
    Build {
        /// Prefix output with flag for immediate execution
        #[clap(
            long,
            short,
            long_help = "When enabled, will prefix the output with a '__IMMEDIATE__'
flag to indicate that the command should be executed. Whatever
integration is set up to handle the output of which-cmd must be
configured to recognize this flag."
        )]
        immediate: bool,
    },
    /// Get a previously built command
    Get,
    /// Generate shell integration code
    Integration {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Troubleshoot configuration
    Doctor,
    /// Get height of TUI, useful for integrations
    Height,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.cmd {
        Commands::Build { immediate } => commands::build_command(immediate)?,
        Commands::Get => commands::get_command()?,
        Commands::Integration { shell } => commands::integration_command(shell)?,
        Commands::Doctor => commands::doctor_command(),
        Commands::Height => commands::height_command(),
    }

    Ok(())
}
