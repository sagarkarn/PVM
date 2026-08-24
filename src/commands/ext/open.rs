use std::path::Path;

pub fn ext_open_command(ext_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
