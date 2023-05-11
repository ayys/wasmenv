use chrono::DateTime;
use directories::BaseDirs;
use dirs::{cache_dir, config_dir, data_dir};

use which::which;

use flate2::read::GzDecoder;
use is_executable::IsExecutable;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{copy, Write};

use std::{env, fs};

use std::path::{PathBuf, Path};
use std::{
    env::consts::{ARCH, OS},
    process::Command,
};
use tar::Archive;

use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub prerelease: bool,
    pub published_at: String,
    pub assets: Vec<ReleaseAsset>,
}

impl Release {
    pub fn version(&self) -> Version {
        let tag_name = self.tag_name.trim_start_matches('v');
        Version::parse(tag_name).unwrap()
    }

    pub fn download_url(&self) -> Option<&str> {
        let filename = get_filename_for_system_architecture(OS, ARCH);
        self.assets.iter().find(|asset| asset.name == filename).map(|asset| &asset.browser_download_url).map(|x| x.as_str())
    }

    pub fn filename(&self) -> Option<String> {
        let filename = get_filename_for_system_architecture(OS, ARCH);
        self.assets.iter().find(|asset| asset.name == filename).map(|asset| format!("{}-{}", self.version(), asset.name))
    }

    pub fn published_time(&self) -> String {
        let date_time = DateTime::parse_from_rfc3339(&self.published_at).unwrap();
        date_time.format("%B %e %Y %r").to_string()
    }

    pub fn tags(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = Vec::new();
        if self.prerelease {
            tags.push("prerelease");
        }
        if let Some(system_wasmer_version) = find_system_wasmer() {
            if system_wasmer_version == self.version() {
                tags.push("system")
            }
        }
        tags
    }
}

/// Fetches the list of releases from the Wasmer GitHub repository and returns them as a vector
/// of `Release` objects.
///
/// # Examples
///
/// ```
/// use wasmerenv::Release;
///
/// let releases = wasmerenv::list_releases().unwrap();
/// for release in releases {
///     println!("{} ({})", release.tag_name, release.published_time());
/// }
/// ```
pub fn list_releases() -> Result<Vec<Release>, reqwest::Error> {
    let url = "https://api.github.com/repos/wasmerio/wasmer/releases";
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).header("User-Agent", "wasmerenv").send()?;
    response.json()
}

pub fn list_releases_interactively() -> Result<Vec<Release>, reqwest::Error> {
    let progress_bar = create_progress_bar(String::from("Fetching wasmer releases..."));
    let releases = list_releases().expect("A list of wasmer releases from github.");
    progress_bar.finish_and_clear();
    Ok(releases)
}

pub fn get_filename_for_system_architecture(target_os: &str, target_arch: &str) -> String {
    let filename = match (target_os, target_arch) {
        ("linux", "x86_64") => "wasmer-linux-amd64.tar.gz",
        ("linux", "aarch64") => "wasmer-linux-aarch64.tar.gz",
        ("linux", "mips64") => "wasmer-linux-mips64.tar.gz",
        ("linux", "riscv64") => "wasmer-linux-riscv64.tar.gz",
        ("macos", "x86_64") => "wasmer-darwin-amd64.tar.gz",
        ("macos", "aarch64") => "wasmer-darwin-arm64.tar.gz",
        ("windows", "x86_64") => "wasmer-windows-amd64.tar.gz",
        ("windows", "gnu") => "wasmer-windows-gnu64.tar.gz",
        ("windows", _) => "wasmer-windows.exe",
        _ => panic!("Unsupported architecture: {}-{}", target_os, target_arch),
    };
    filename.to_string()
}


fn version_from_version_string(version_string: String) -> anyhow::Result<Version> {
    match version_string
        .trim()
        .trim_start_matches("wasmer ")
        .parse::<Version>() {
            Ok(version) => {
                Ok(version)
            },
            Err(_) => {
                Err(anyhow::anyhow!("Could not get wasmer version form the version string"))
            }
        }
}

/// Searches for the system Wasmer binary and returns its version.
///
/// Returns `None` if Wasmer is not installed or the installed version is not compatible.
pub fn find_system_wasmer() -> Option<Version> {
    // Try to locate the Wasmer binary in the user's home directory.
    let wasmer_path = BaseDirs::new().map(|base_dirs| {
        let wasmer_path = base_dirs.home_dir().join(".wasmer/bin/wasmer");
        if wasmer_path.is_executable() {
            Some(wasmer_path)
        } else {
            None
        }
    })?;

    if let Some(wasmer_path) = wasmer_path {
        let output = Command::new(wasmer_path).arg("--version").output().ok()?;
        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout).to_string();
            if let Ok(version) = version_from_version_string(version_str) {
                return Some(version);
            } else {
                return None;
            }

        }
    }
    None
}

/// Searches for the system Wasmer binary and returns its version.
///
/// Returns `None` if Wasmer is not installed or the installed version is not compatible.
pub fn find_current_wasmer() -> Option<Version> {
    let output = Command::new("wasmer").arg("--version").output().ok()?;
    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout).to_string();
        if let Ok(version) = version_from_version_string(version_str) {
            return Some(version);
        }
    }
    None
}

/// Finds the location of current wasmer executable
pub fn find_current_wasmer_dir() -> anyhow::Result<PathBuf> {
    Ok(which("wasmer")?.parent().expect("path to wasmer executable").to_path_buf())
}


pub fn download_wasmer_to_cache(release: &Release) -> anyhow::Result<PathBuf> {
    let url = release
        .download_url()
        .expect("Download url for wasmer release");
    let filename = release.filename().expect("Filename for wasmer release");
    let filepath = cache_dir()
        .ok_or(anyhow::anyhow!("Could not get cache directory"))?
        .join("wasmerenv")
        .join(filename);

    if filepath.exists() {
        return Ok(filepath);
    }
    println!("downloading to {}", filepath.to_str().unwrap());

    create_dir_all(filepath.parent().unwrap())?;

    let client = reqwest::blocking::Client::new();
    let progress_bar = create_progress_bar(format!("Downloading wasmer {}...", release.version()));

    let mut response = &mut client.get(url).send()?;

    let mut tmp_file = File::create(&filepath)?;

    copy(&mut response, &mut tmp_file)?;
    progress_bar.finish_and_clear();

    Ok(filepath)
}

pub fn download_and_install_wasmer(release: &Release, dest_dir: &PathBuf) -> anyhow::Result<()> {
    let filepath = download_wasmer_to_cache(release)?;

    let progress_bar = create_progress_bar(format!("Installing wasmer {}...", release.version()));

    if !dest_dir.exists() {
        std::fs::create_dir_all(dest_dir)?;
    }

    let file = File::open(filepath)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    archive.unpack(dest_dir)?;
    progress_bar.finish_and_clear();

    Ok(())
}


fn create_progress_bar(message: String) -> ProgressBar {
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(ProgressStyle::default_spinner().tick_strings(&[
        "( ●    )",
        "(  ●   )",
        "(   ●  )",
        "(    ● )",
        "(     ●)",
        "(    ● )",
        "(   ●  )",
        "(  ●   )",
    ]));
    progress_bar.set_message(message);
    progress_bar.enable_steady_tick(Duration::from_millis(100));

    progress_bar
}

fn create_config_files(config_dir: &Path, wasmer_current_dir: &str) -> anyhow::Result<()> {
    let filepath = config_dir.join("wasmerenv.sh");
    if !filepath.exists() {
        fs::create_dir_all(config_dir)?;
        let mut wasmerenv_sh = File::create(filepath)?;
        let wasmerenv_sh_contents = format!(
            "\
            # wasmer config\n\
            export WASMER_DIR=\"{}\"\n\
            export PATH=\"$WASMER_DIR/bin\":$PATH\n",
            wasmer_current_dir
        );
        wasmerenv_sh.write_all(wasmerenv_sh_contents.as_bytes())?;
    }
    let filepath = config_dir.join("wasmerenv.fish");
    if !filepath.exists() {
        fs::create_dir_all(config_dir)?;
        let mut wasmerenv_sh = File::create(filepath)?;
        let wasmerenv_sh_contents = format!(
            "\
            # wasmer config for fish\n\
            set -x WASMER_DIR \"{}\"\n\
            set -x PATH $WASMER_DIR/bin $PATH\n",
            wasmer_current_dir
        );
        wasmerenv_sh.write_all(wasmerenv_sh_contents.as_bytes())?;
    }

    Ok(())
}


/// returns path to wasmerenv config directory
pub fn wasmerenv_config_dir() -> anyhow::Result<PathBuf> {
    let (config_dir, _) = setup_config_directory()?;
    Ok(config_dir)
}

fn setup_config_directory() -> anyhow::Result<(PathBuf, PathBuf)> {
    let config_dir = config_dir()
        .expect("Config directory should be present")
        .join("wasmerenv");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    let data_dir = data_dir().expect("Data directory should be present");
    let wasmer_current_dir = data_dir.join("wasmerenv/current");
    if !wasmer_current_dir.exists() {
        fs::create_dir_all(&wasmer_current_dir)?;
    }

    create_config_files(&config_dir, wasmer_current_dir.to_str().expect("String containing wasmer current path"))?;

    Ok((config_dir, wasmer_current_dir))
}



/// check if WASMERENV_DIR exists, because that means wasmerenv has been properly setup
pub fn verify_wasmerenv_is_in_path() -> anyhow::Result<()> {
    match env::var("WASMERENV_DIR") {
        Ok(_) => {
            Ok(())
        }
        Err(_) => {
            Err(anyhow::anyhow!(
                "Looks like you haven't initialized wasmerenv.\n\
                run `wasmerenv shell | source` to initialize it.\n"
            ))
        }
    }
}

pub fn release_to_install(version: &Option<VersionReq>) -> anyhow::Result<Option<Release>> {
    let releases = list_releases_interactively()?;

    let release = if let Some(req) = version {
        releases.into_iter().find(|rel| req.matches(&rel.version()))
    } else {
        releases.first().cloned()
    };
    Ok(release)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_wasmerenv_config_dir() -> anyhow::Result<()> {
        let result = wasmerenv_config_dir()?;
        assert!(result.exists());
        assert!(result.ends_with("wasmerenv"));
        Ok(())
    }

    #[test]
    fn test_list_releases() -> anyhow::Result<()> {
        let releases = list_releases()?;
        assert!(!releases.is_empty());
        Ok(())
    }

    #[test]
    fn test_verify_wasmerenv_is_in_path() {
        // Test the case where WASMERENV_DIR is set
        env::set_var("WASMERENV_DIR", "/path/to/wasmerenv");
        assert!(verify_wasmerenv_is_in_path().is_ok());

        // Test the case where WASMERENV_DIR is not set
        env::remove_var("WASMERENV_DIR");
        let result = verify_wasmerenv_is_in_path();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Looks like you haven't initialized wasmerenv.\n\
            run `wasmerenv shell | source` to initialize it.\n"
        );
    }

    #[test]
    fn test_version_from_version_string() {
        // Test the case where the version string is valid
        let version_string = "wasmer 1.0.0".to_string();
        let version = version_from_version_string(version_string).unwrap();
        assert_eq!(version.to_string(), "1.0.0");

        // Test the case where the version string is invalid
        let version_string = "invalid version string".to_string();
        let result = version_from_version_string(version_string);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Could not get wasmer version form the version string"
        );
    }
}
