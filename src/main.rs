mod command_node;
mod config;
mod tui;

use crate::config::Config;
use clap::{Arg, ArgAction, Command};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("which-cmd")
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

    let config = match Config::from_file("fixtures/commands.yml") {
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
