use std::{env::temp_dir, path::PathBuf, process::Command};

use semver_eq::VersionReq;

use crate::utils::{
    download_and_install_wasmer, find_current_wasmer, find_current_wasmer_dir, release_to_install,
};

fn setup_exec(version: Option<VersionReq>, install_prerelease: bool) -> anyhow::Result<PathBuf> {
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
    if let Some(release) = release_to_install(&Some(version), install_prerelease)? {
        download_and_install_wasmer(&release, &dest_dir)?;
    };

    Ok(dest_dir)
}

pub fn exec(
    version: Option<VersionReq>,
    command: Vec<String>,
    install_prerelease: bool,
) -> anyhow::Result<()> {
    let dest_dir = setup_exec(version, install_prerelease)?.join("wasmer");
    let wasmer_exe = dest_dir.to_str().unwrap();

    let output = Command::new(wasmer_exe).args(command).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    print!("{}", stdout);
    eprint!("{}", stderr);
    Ok(())
}
