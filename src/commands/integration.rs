pub fn integration_command(shell: &str) -> Result<(), Box<dyn std::error::Error>> {
    match shell {
        "zsh" => {
            println!(
                r#"
# which-cmd integration for zsh
which_cmd_widget() {{
    local result
    <$TTY which-cmd build
    if [[ $? -eq 0 ]]; then
        result=$(which-cmd get)
        LBUFFER+="$result"
    fi
    zle reset-prompt
}}
zle -N which_cmd_widget
bindkey '^P' which_cmd_widget
"#
            );
        }
        "zsh-tmux" => {
            println!(
                r#"
# which-cmd integration for zsh + tmux
which_cmd_tmux_widget() {{
  if [[ $LBUFFER == "" ]]; then
    local result
    tmux display-popup -S fg=brightblack -T '#[fg=white bold] which-cmd #[fg=default]' -y S -w 95% -h 12 -b rounded -E "which-cmd build --immediate"
    result=$(which-cmd get)
    if [[ $result != "" ]]; then
      if [[ $result = __IMMEDIATE__* ]]; then
        local cmd
        cmd=$(echo $result | cut -d' ' -f2-)
        LBUFFER+="$cmd"
        zle accept-line
      else
        LBUFFER+="$result"
        zle self-insert
      fi
    fi
    zle reset-prompt
  else
    zle self-insert
  fi
}}
zle -N which_cmd_tmux_widget
bindkey ' ' which_cmd_tmux_widget
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
