use std::fs;

use crate::config::Config;
use crate::options::Options;
use crate::tui;

use crate::constants::*;

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
            let xdg_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;
            let output_path = xdg_dirs.place_data_file(OUTPUT_FILE_NAME)?;
            fs::write(output_path, command)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
