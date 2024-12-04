use crate::config::Config;

pub fn doctor_command() {
    let _ = match Config::from_file() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    println!("Configuration file is valid.");
}
