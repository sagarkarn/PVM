use std::{collections::HashSet, fs, path::Path, process::Command};

use crate::db::PhpVersion;

pub fn list_ext_command(
    ext_path: &Path,
    php_version: &PhpVersion,
) -> Result<(), Box<dyn std::error::Error>> {
    let php_exe_path = Path::new(&php_version.path).join("php.exe");
    let output = Command::new(php_exe_path).arg("-m").output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Ok(());
    }

    let loaded: HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip_while(|line| line.trim() != "[PHP Modules]")
        .skip(1)
        .take_while(|line| line.trim() != "[Zend Modules]")
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_lowercase)
        .collect();

    for entry in fs::read_dir(ext_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if !file_name.starts_with("php_") || !file_name.ends_with(".dll") {
            continue;
        }

        let extension = file_name
            .strip_prefix("php_")
            .unwrap()
            .strip_suffix(".dll")
            .unwrap()
            .to_lowercase();

        let status = if loaded.contains(&extension) {
            "loaded"
        } else {
            "disabled"
        };

        println!("{:<20} {}", extension, status);
    }

    Ok(())
}
