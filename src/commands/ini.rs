use crate::commands::PvmContext;
use std::fs;
use std::path::Path;

/// Open php.ini in Notepad.
pub fn ini_command(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    let current_version = match ctx.db.get_current_php_version()? {
        Some(v) => v,
        None => {
            println!("No current version found");
            return Ok(());
        }
    };

    let ini_path = Path::new(&current_version.path).join("php.ini");
    if !ini_path.exists() {
        println!("php.ini not found copying php.ini-development file");
        let dev_path = Path::new(&current_version.path).join("php.ini-development");
        if !dev_path.exists() {
            println!("php.ini-development not found");
            return Ok(());
        }
        fs::copy(&dev_path, &ini_path)?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("notepad.exe")
            .arg(&ini_path)
            .spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!(
            "Opening notepad is only supported on Windows. File path: {}",
            ini_path.to_string_lossy()
        );
    }
    Ok(())
}
