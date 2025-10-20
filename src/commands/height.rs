use crate::options::Options;

pub fn height_command() {
    // Return the default TUI height for shell integrations
    let default_opts = Options::default();
    println!("{}", default_opts.height)
}
