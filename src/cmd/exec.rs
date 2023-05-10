use std::{process::Command, env::temp_dir, path::PathBuf};

use semver::VersionReq;

use crate::utils::{find_current_wasmer, download_and_install_wasmer, release_to_install, find_current_wasmer_dir};



fn setup_exec(version: Option<VersionReq>) -> anyhow::Result<PathBuf> {
    let current = find_current_wasmer();
    let dest_dir = find_current_wasmer_dir()?;

    if version.is_none() {
        return Ok(dest_dir);
    }
    let version = version.unwrap();
    if let Some(current_version) = current {
        if version.matches(&current_version) {
            return Ok(dest_dir);
        }
    }
    let dest_dir = temp_dir();
    if let Some(release) = release_to_install(&Some(version))?  {
        download_and_install_wasmer(&release, &dest_dir)?;
    };

    Ok(dest_dir)
}


pub fn exec(version: Option<VersionReq>, command: Vec<String>) -> anyhow::Result<()> {
    let dest_dir = setup_exec(version)?.join("wasmer");
    let wasmer_exe = dest_dir.to_str().unwrap();

    let output = Command::new(wasmer_exe).args(command).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    print!("{}", stdout);
    eprint!("{}", stderr);
    Ok(())
}
