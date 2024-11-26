use crate::config::Config;
use crate::options::Options;
use crate::tui;

pub fn build_command(immediate: bool) -> Result<(), Box<dyn std::error::Error>> {
    let opts = Options {
        print_immediate_tag: immediate,
    };

    let config = match Config::from_file() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    match tui::run_tui(config, opts) {
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
