use crate::commands::PvmContext;
use std::path::Path;

/// Open extension folder in file explorer.
pub fn ext_command(
    ctx: &PvmContext,
    version: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let php_version = match version {
        Some(ver) => ctx.db.get_php_version_exact(&ver)?,
        None => ctx.db.get_current_php_version()?,
    };

    let php_version = match php_version {
        Some(v) => v,
        None => {
            println!("Version not found");
            return Ok(());
        }
    };

    let ext_path = Path::new(&php_version.path).join("ext");
    if !ext_path.exists() {
        println!("ext not found");
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(&ext_path)
            .spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!(
            "Opening explorer is only supported on Windows. Directory path: {}",
            ext_path.to_string_lossy()
        );
    }
    Ok(())
}
