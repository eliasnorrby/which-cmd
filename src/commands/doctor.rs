use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::{
    condition::ConditionEvaluator, config::Config, constants::LOCAL_CONFIG_FILE_NAMES,
    search::get_search_options,
};

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

    // Check for local config files
    if let Ok(cwd) = std::env::current_dir() {
        for name in LOCAL_CONFIG_FILE_NAMES {
            let path = cwd.join(name);
            if path.exists() {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if metadata.permissions().mode() & 0o111 == 0 {
                        eprintln!(
                            "Warning: local config '{}' found but not executable. Run: chmod +x {}",
                            name, name
                        );
                    } else {
                        println!("Local config '{}' found and executable.", name);
                    }
                }
                break;
            }
        }
    }

    println!("Configuration file is valid.");
}
