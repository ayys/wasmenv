use std::{env, path::PathBuf};

use anyhow::anyhow;

use crate::utils::wasmerenv_config_dir;

pub fn shell(name: Option<String>) -> anyhow::Result<()> {
    let env_shell = env::var("SHELL").unwrap();
    let shell_path = match name {
        Some(ref shell) => shell,
        None => &env_shell,
    };
    let shell_name = PathBuf::from(shell_path)
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or(anyhow!("Invalid shell name"))?
        .to_owned();
    let wasmerenv_dir = wasmerenv_config_dir()?;
    let wasmerenv_dir = wasmerenv_dir.to_str().unwrap();
    let shell_code = match shell_name.as_str() {
        "bash" | "zsh" =>
            format!(
                r#"
# {shell_name} config for wasmerenv
# copy this to ~/.{shell_name}rc
export WASMERENV_DIR="{0}"
[ -s "{0}/wasmerenv.sh" ] && source "{0}/wasmerenv.sh"
"#,
                wasmerenv_dir
            ),
        "fish" =>
            format!(
                r#"
# {shell_name} config for wasmerenv
# Copy this to ~/.config/fish/config.fish
set -x WASMER_DIR "{0}"
set -x PATH $WASMER_DIR/bin $PATH
"#,
                wasmerenv_dir
            ),
        _ => {
            return Err(anyhow!(format!("Shell `{}` not recognized. Try one of `bash`, `zsh` or `fish`", shell_name)));
        }
    };
    println!("{}", shell_code);
    // let mut stdout = stdout();
    // stdout.write_all(shell_code.as_bytes())?;
    Ok(())
}
