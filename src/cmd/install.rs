use std::{env, fs};

use anyhow;

use dirs::home_dir;
use semver::VersionReq;

use crate::utils::{
    download_and_install_wasmer, find_current_wasmer, list_releases_interactively,
    verify_wasmerenv_is_in_path,
};

// - DONE download the tar.gz file into `~/.wasmerenv/cache/` directory
// - extract this file to `~/.wasmerenv/current` directory
// - set environment variable WASMER_DIR=`~/.wasmerenv/current`
// - run `source $WASMER_DIR/wasmer.sh`

pub fn install(version: Option<VersionReq>) -> anyhow::Result<()> {
    verify_wasmerenv_is_in_path()?;
    let current_version = find_current_wasmer().unwrap();
    if let Some(ref ver) = version {
        if ver.matches(&current_version) {
            println!("You are already using wasmer {}", ver);
            return Ok(());
        }
    }
    let releases = list_releases_interactively().expect("A list of wasmer versions from github.");
    let release = if let Some(ref version) = version {
        releases
            .iter()
            .find(|release| version.matches(&release.version()))
    } else {
        // version is None so install the latest version
        Some(&releases[0])
    };

    if let Some(release) = release {
        if release.version() == current_version {
            println!(
                "You're already using wasmer {}, which is the latest version.",
                current_version
            );
            return Ok(());
        }

        let home_dir = home_dir().expect("Could not get home directory");
        let wasmer_current_dir = home_dir.join(".wasmerenv/current");
        let wasmer_old_dir = home_dir.join(".wasmerenv/old");
        if wasmer_old_dir.exists() {
            fs::remove_dir_all(&wasmer_old_dir)?;
        }
        if wasmer_current_dir.exists() {
            fs::rename(&wasmer_current_dir, &wasmer_old_dir)?;
        }
        if download_and_install_wasmer(release, &wasmer_current_dir).is_err() && wasmer_current_dir.exists() && wasmer_old_dir.exists() {
            fs::rename(&wasmer_old_dir, &wasmer_current_dir)?;
            println!("Failed to install wasmer. Reverting back to the old version.");
        };
        env::set_var("WASMER_DIR", &wasmer_current_dir);
        println!("You are now using wasmer {}. You can run `wasmer --version` to check your version of wasmer.", release.version());
    } else {
        println!(
            "You are trying to install wasmer {}, but it does not exist.
To install the latest version, run `wasmerenv use`.",
            version.unwrap(),
        )
    }

    Ok(())
}
