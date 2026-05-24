use crate::commands::PvmContext;
use std::fs;
use std::path::Path;

/// Uninstall/remove a registered PHP version.
pub fn uninstall_command(
    ctx: &PvmContext,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let php_version = ctx.db.get_php_version(version)?;
    let php_version = match php_version {
        Some(v) => v,
        None => {
            println!("Version {} not found", version);
            return Ok(());
        }
    };

    if php_version.is_current {
        println!(
            "Version {} is currently in use. Switch to another version before uninstalling.",
            php_version.version
        );
        return Ok(());
    }

    let path = Path::new(&php_version.path);
    if path.exists() {
        fs::remove_dir_all(path)?;
        println!("Deleted folder: {}", php_version.path);
    } else {
        println!("Folder not found: {}", php_version.path);
    }

    ctx.db.remove_php_versions_by_name(&php_version.version)?;
    println!("Uninstalled version {}", php_version.version);
    Ok(())
}
