use std::{env, fs};

use anyhow;

use dirs::data_dir;
use semver_eq::VersionReq;

use crate::utils::{
    download_and_install_wasmer, find_current_wasmer, release_to_install,
    verify_wasmenv_is_in_path, Release,
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
    let current_version = find_current_wasmer();
    let match_with_current_version = current_version.is_some();
    if match_with_current_version {
        if let Some(ref ver) = version {
            if ver.matches(&current_version.unwrap()) {
                return Err(anyhow::anyhow!("You are already using wasmer {}", ver));
            }
        }
    }

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

    let data_dir = data_dir().expect("Could not get home directory");
    let version = release.version().to_string();
    let wasmer_dir_path = format!("wasmenv/{version}");
    let wasmer_version_dir = data_dir.join(wasmer_dir_path);
    let wasmer_current_dir = data_dir.join("wasmenv/current");
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
    let parent_dir = current_wasmer.parent().unwrap();
    fs::create_dir_all(parent_dir)?;
    symlink::symlink_file(versioned_wasmer, current_wasmer)?;
    symlink::symlink_file(
        versioned_wasmer,
        wasmer_current_dir.join(format!("bin/wasmer{version}")),
    )?;
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
