use clap::ValueEnum;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Shell {
    Zsh,
    ZshTmux,
    BashTmux,
}

pub fn integration_command(shell: Shell) -> Result<()> {
    match shell {
        Shell::Zsh => {
            println!(
                r#"
# which-cmd integration for zsh
which_cmd_widget() {{
    local result
    # The <$TTY part ensures that which-cmd reads input from the terminal ($TTY) rather than from
    #   the shell's standard input, which may not be connected to the terminal when running in a
    #   ZLE widget.
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
        Shell::ZshTmux => {
            println!(
                r#"
# which-cmd integration for zsh + tmux
which_cmd_tmux_widget() {{
  if [[ $LBUFFER == "" ]]; then
    local result
    tmux display-popup -S fg=brightblack -T '#[fg=white bold] which-cmd #[fg=default]' -y P -w 95% -h 12 -b rounded -E "which-cmd build --immediate"
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
        Shell::BashTmux => {
            println!(
                r#"
# which-cmd integration for bash + tmux
which_cmd_tmux_widget() {{
  if [[ "$READLINE_LINE" == "" ]]; then
    local result
    tmux display-popup -S fg=brightblack -T '#[fg=white bold] which-cmd #[fg=default]' -y P -w 95% -h 12 -b rounded -E "which-cmd build --immediate"
    result=$(which-cmd get)
    if [[ "$result" != "" ]]; then
      if [[ "$result" = __IMMEDIATE__* ]]; then
        local cmd
        cmd=$(echo "$result" | cut -d' ' -f2-)
        READLINE_LINE="$cmd"
        READLINE_POINT=${{#READLINE_LINE}}
        # Simulate pressing Enter by inserting newline
        eval "$READLINE_LINE"
        READLINE_LINE=""
        READLINE_POINT=0
      else
        READLINE_LINE+="$result"
        READLINE_POINT=${{#READLINE_LINE}}
      fi
    fi
  fi
}}

which_cmd_tmux_space() {{
  if [[ "$READLINE_LINE" == "" ]]; then
    which_cmd_tmux_widget
  else
    READLINE_LINE+=" "
    READLINE_POINT=${{#READLINE_LINE}}
  fi
}}
bind -x '"\x20": which_cmd_tmux_space'
"#
            );
        }
    }
    Ok(())
}
