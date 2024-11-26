pub fn integration_command(shell: &str) -> Result<(), Box<dyn std::error::Error>> {
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
