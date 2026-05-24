use crate::commands::PvmContext;
use std::path::Path;

/// Add local php version to PVM.
pub fn add_command(
    ctx: &PvmContext,
    version: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if ctx.db.get_php_version_exact(version)?.is_some() {
        println!("Version {} already exists", version);
        return Ok(());
    }

    let moved_path = ctx.base_dir.join(format!("php{}", version));
    let src_path = Path::new(path);
    if !src_path.exists() {
        return Err(format!("Source path '{}' does not exist", path).into());
    }

    crate::helpers::copy_dir_all(src_path, &moved_path)?;
    ctx.db
        .add_php_version(version, &moved_path.to_string_lossy(), false)?;
    println!("Added version {}", version);
    Ok(())
}
