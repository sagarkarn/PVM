use std::path::Path;

use anyhow::Result;

pub fn ext_open_command(ext_path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(&ext_path)
            .spawn()
            .unwrap();
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
