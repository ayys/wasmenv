use crate::utils::{find_current_wasmer, find_current_wasmer_dir, verify_wasmenv_is_in_path};

pub fn current(verbose: bool) -> anyhow::Result<()> {
    verify_wasmenv_is_in_path()?;
    let current_version = find_current_wasmer();
    println!("wasmer {}", current_version.unwrap());

    if !verbose {
        return Ok(());
    }
    if let Some(path) = find_current_wasmer_dir()?.to_str() {
        println!("Installed at: {}", path);
    }

    Ok(())
}
