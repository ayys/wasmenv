use std::fs::remove_dir_all;

use anyhow::Context;

use crate::utils::{verify_wasmenv_is_in_path, wasmenv_cache_dir};

pub fn clear_cache() -> anyhow::Result<()> {
    verify_wasmenv_is_in_path()?;
    let cache_dir = wasmenv_cache_dir().context("Could not get cache dir")?;

    remove_dir_all(&cache_dir).context("Could not clear cache directory")?;
    println!("cleared {:?}", cache_dir);
    Ok(())
}
