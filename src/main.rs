mod command_node;
mod config;
mod tui;

use crate::config::Config;

fn main() {
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
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
