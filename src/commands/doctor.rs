use std::path::Path;

use crate::{condition::ConditionEvaluator, config::Config, search::get_search_options};

pub fn doctor_command(config_path: Option<&Path>) {
    let config = match config_path {
        Some(path) => Config::from_path(path),
        None => Config::from_file(),
    };

    let config = match config {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    let mut evaluator = ConditionEvaluator::new();
    let search_options = get_search_options(&config.keys, &mut evaluator);

    if search_options.iter().any(|n| n.id.contains('/')) {
        eprintln!("Warning: found node bound to the '/' character, search will be unavailable.");
    }

    println!("Configuration file is valid.");
}
