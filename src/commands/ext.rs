use crate::commands::PvmContext;
use clap::Subcommand;
use std::{collections::HashSet, fs, path::Path, process::Command};

#[derive(Subcommand)]
pub enum ExtCommand {
    Open,
    List,
}

/// Open extension folder in file explorer.
pub fn ext_command(
    ctx: &PvmContext,
    version: Option<String>,
    command: Option<ExtCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    let php_version = match version {
        Some(ver) => ctx.db.get_php_version_exact(&ver)?,
        None => ctx.db.get_current_php_version()?,
    };

    let command = match command {
        Some(cmd) => cmd,
        None => ExtCommand::Open,
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

    match command {
        ExtCommand::Open => open_ext_folder(&ext_path)?,
        ExtCommand::List => {
            let php_exe_path = Path::new(&php_version.path).join("php.exe");
            list_ext_folder(&ext_path, &php_exe_path)?;
        }
    }
    Ok(())
}

fn open_ext_folder(ext_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

fn list_ext_folder(ext_path: &Path, php_exe_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(php_exe_path).arg("-m").output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Ok(());
    }

    let loaded: HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip_while(|line| *line != "[PHP Modules]")
        .skip(1)
        .take_while(|line| *line != "[Zend Modules]")
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
