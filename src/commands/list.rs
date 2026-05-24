use crate::commands::PvmContext;

/// List all installed php versions.
pub fn list_command(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Current working directory: {}",
        ctx.base_dir.to_string_lossy()
    );
    let versions = ctx.db.get_php_versions()?;
    if versions.is_empty() {
        println!("No versions found");
        return Ok(());
    }

    for v in versions {
        let current_str = if v.is_current { "(current)" } else { "" };
        println!("{} - {} {}", v.version, v.path, current_str);
    }
    Ok(())
}
