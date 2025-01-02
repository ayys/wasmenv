use std::{env, fs};

use anyhow::{self, Context};

use semver_eq::VersionReq;

use crate::utils::{
    download_and_install_wasmer, find_current_wasmer, release_to_install,
    verify_wasmenv_is_in_path, wasmenv_data_dir, Release,
};

fn check_release_already_installed(release: &Release) -> anyhow::Result<()> {
    let current_version = find_current_wasmer();
    if let Some(current_version) = current_version {
        if release.version() == current_version {
            return Err(anyhow::anyhow!(
                "You're already using wasmer {}, which is the latest version.",
                current_version
            ));
        }
    }
    Ok(())
}

fn install_version(
    version: Option<VersionReq>,
    install_prerelease: bool,
) -> anyhow::Result<Release> {
    verify_wasmenv_is_in_path()?;

    let release = match release_to_install(&version, install_prerelease)? {
        Some(rel) => rel,
        None => {
            return Err(anyhow::anyhow!(
                "Wasmer release `{}` was not found",
                version.unwrap()
            ))
        }
    };
    check_release_already_installed(&release)?;

    let data_dir = wasmenv_data_dir().expect("Getting wasmenv data directory");
    let version = release.version().to_string();
    let wasmer_dir_path = &version;
    let wasmer_version_dir = data_dir.join(wasmer_dir_path);
    let wasmer_current_dir = data_dir.join("current");
    let wasmer_old_dir = data_dir.join(".wasmenv/old");

    if download_and_install_wasmer(&release, &wasmer_version_dir).is_err()
        && wasmer_version_dir.exists()
        && wasmer_old_dir.exists()
    {
        fs::rename(&wasmer_old_dir, &wasmer_current_dir)?;
        println!("Failed to install wasmer. Reverting back to the old version.");
    };

    let current_wasmer = &wasmer_current_dir.join("bin/wasmer");
    let versioned_wasmer = &wasmer_version_dir.join("bin/wasmer");

    if current_wasmer.exists() {
        fs::remove_file(current_wasmer)?;
    }

    fs::create_dir_all(&wasmer_current_dir)?;

    // make sure the current wasmer directory exists
    let parent_dir = current_wasmer
        .parent()
        .context("Find the parent of current wasmer dir")?;
    fs::create_dir_all(parent_dir)?;
    symlink::symlink_file(versioned_wasmer, current_wasmer)?;

    let wasmer_versioned_path = wasmer_current_dir.join(format!("bin/wasmer{version}"));
    if wasmer_versioned_path.exists() {
        fs::remove_file(&wasmer_versioned_path).context("Removing versioned path")?;
    }

    symlink::symlink_file(versioned_wasmer, wasmer_versioned_path)?;
    env::set_var("WASMER_DIR", &wasmer_current_dir);
    Ok(release)
}

pub fn install(version: Option<VersionReq>, install_prerelease: bool) -> anyhow::Result<()> {
    let release = install_version(version, install_prerelease)?;
    println!(
                "You are now using wasmer {}. You can run `wasmer --version` to check your version of wasmer.",
                release.version()
            );

    Ok(())
}
