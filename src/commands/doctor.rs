use crate::{config::Config, search::get_search_options};

pub fn doctor_command() {
    let config = match Config::from_file() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    let search_options = get_search_options(&config.keys);

    if search_options.iter().find(|n| n.id.contains('/')).is_some() {
        eprintln!("Warning: found node bound to the '/' character, search will be unavailable.");
    }

    println!("Configuration file is valid.");
}
