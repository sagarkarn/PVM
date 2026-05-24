use crate::commands::PvmContext;
use std::fs;
use std::path::Path;

/// Switch to use the specified php version.
pub fn use_command(ctx: &PvmContext, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let php_version = ctx.db.get_php_version(version)?;
    let php_version = match php_version {
        Some(v) => v,
        None => {
            println!("Version {} not found", version);
            return Ok(());
        }
    };

    if php_version.is_current {
        println!("Version {} is already in use", php_version.version);
        return Ok(());
    }

    let target_path = Path::new(&php_version.path);
    println!("Using version {}", php_version.path);
    if !target_path.exists() {
        println!("Path {} not found", php_version.path);
        return Ok(());
    }

    // 1. Move currently active php version back to php<version>
    if let Some(current_version) = ctx.db.get_current_php_version()? {
        let current_path = Path::new(&current_version.path);
        let moved_path = ctx.base_dir.join(format!("php{}", current_version.version));
        println!(
            "Moving {} to {}",
            current_version.path,
            moved_path.to_string_lossy()
        );
        if current_path.exists() {
            fs::rename(current_path, &moved_path)?;
        }
        println!(
            "Moved {} to {}",
            current_version.path,
            moved_path.to_string_lossy()
        );
        ctx.db.update_php_version_path_and_current(
            current_version.id,
            &moved_path.to_string_lossy(),
            false,
        )?;
    }

    // 2. Move target version directory to active php directory
    let active_php_dir = ctx.base_dir.join("php");
    fs::rename(target_path, &active_php_dir)?;
    ctx.db.update_php_version_path_and_current(
        php_version.id,
        &active_php_dir.to_string_lossy(),
        true,
    )?;
    println!("Using version {}", php_version.version);
    crate::commands::setup_command(ctx)?;
    Ok(())
}
