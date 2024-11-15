mod command_node;
mod config;
mod tui;

use crate::config::Config;
use clap::{Arg, ArgAction, Command};
use std::error::Error;

const COMMAND_NAME: &str = "which-cmd";
const CONFIG_FILE_NAME: &str = "commands.yml";

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new(COMMAND_NAME)
        .version("0.1.0")
        .about("Command Builder Tool")
        .arg(
            Arg::new("shell")
                .long("shell")
                .value_name("SHELL")
                .help("Generate shell integration code")
                .action(ArgAction::Set),
        )
        .get_matches();

    if let Some(_) = matches.value_source("shell") {
        let shell = matches.get_one::<String>("shell").unwrap();
        print_shell_integration(shell)?;
        return Ok(());
    }

    let config_dirs = xdg::BaseDirectories::with_prefix(COMMAND_NAME)?;
    let config_path = match config_dirs.find_config_file(CONFIG_FILE_NAME) {
        Some(path) => path,
        None => {
            eprintln!(
                "Configuration file not found at {}",
                config_dirs.place_config_file(CONFIG_FILE_NAME)?.display()
            );
            std::process::exit(1);
        }
    };

    let config = match Config::from_file(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    match tui::run_tui(config) {
        Ok(command) => {
            println!("{}", command);
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn print_shell_integration(shell: &str) -> Result<(), Box<dyn Error>> {
    match shell {
        "zsh" => {
            println!(
                r#"
# which-cmd Integration for zsh
which_cmd_widget() {{
    local result
    result=$(<$TTY which-cmd)
    if [[ $? -eq 0 ]]; then
        LBUFFER+="$result"
    fi
    zle reset-prompt
}}
zle -N which_cmd_widget
bindkey '^P' which_cmd_widget
"#
            );
        }
        _ => {
            eprintln!("Shell '{}' is not supported.", shell);
            std::process::exit(1);
        }
    }
    Ok(())
}
