use crate::utils::find_current_wasmer;

pub fn current() -> anyhow::Result<()> {
    let current_version = find_current_wasmer();
    println!("wasmer {}", current_version.unwrap());
    Ok(())
}
