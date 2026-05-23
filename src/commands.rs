use std::fs;
use std::path::{Path, PathBuf};
use crate::db::Db;

pub struct PvmContext {
    pub base_dir: PathBuf,
    pub db: Db,
}

/// Add local php version to PVM.
pub fn add_command(ctx: &PvmContext, version: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    ctx.db.add_php_version(version, &moved_path.to_string_lossy(), false)?;
    println!("Added version {}", version);
    Ok(())
}

/// List all installed php versions.
pub fn list_command(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    println!("Current working directory: {}", ctx.base_dir.to_string_lossy());
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
        println!("Moving {} to {}", current_version.path, moved_path.to_string_lossy());
        if current_path.exists() {
            fs::rename(current_path, &moved_path)?;
        }
        println!("Moved {} to {}", current_version.path, moved_path.to_string_lossy());
        ctx.db.update_php_version_path_and_current(current_version.id, &moved_path.to_string_lossy(), false)?;
    }

    // 2. Move target version directory to active php directory
    let active_php_dir = ctx.base_dir.join("php");
    fs::rename(target_path, &active_php_dir)?;
    ctx.db.update_php_version_path_and_current(php_version.id, &active_php_dir.to_string_lossy(), true)?;
    println!("Using version {}", php_version.version);
    Ok(())
}

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
        println!("Opening notepad is only supported on Windows. File path: {}", ini_path.to_string_lossy());
    }
    Ok(())
}

/// Open extension folder in file explorer.
pub fn ext_command(ctx: &PvmContext, version: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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
        println!("Opening explorer is only supported on Windows. Directory path: {}", ext_path.to_string_lossy());
    }
    Ok(())
}

/// Enable extension in active php.ini.
pub fn ext_enable_command(ctx: &PvmContext, ext: &str) -> Result<(), Box<dyn std::error::Error>> {
    let active_php_dir = ctx.base_dir.join("php");
    let ext_dir = active_php_dir.join("ext");
    let dll_name = format!("php_{}.dll", ext);
    let ext_dll = ext_dir.join(&dll_name);
    if !ext_dll.exists() {
        println!("file not found");
        return Ok(());
    }

    let php_ini_path = active_php_dir.join("php.ini");
    if !php_ini_path.exists() {
        println!("php.ini not found copying php.ini-development file");
        let dev_path = active_php_dir.join("php.ini-development");
        if !dev_path.exists() {
            println!("php.ini-development not found");
            return Ok(());
        }
        fs::copy(&dev_path, &php_ini_path)?;
    }

    let content = fs::read_to_string(&php_ini_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let target_enabled = format!("extension={}", ext);
    let target_disabled = format!(";extension={}", ext);

    if lines.iter().any(|l| l.trim() == target_enabled) {
        println!("extension already enabled");
        return Ok(());
    }

    if let Some(index) = lines.iter().position(|l| l.trim() == target_disabled) {
        lines[index] = target_enabled;
        fs::write(&php_ini_path, lines.join("\r\n"))?;
        println!("extension enabled");
    } else {
        println!("extension not found in ini file");
    }
    Ok(())
}

/// Install PHP from windows.php.net.
pub fn install_command(ctx: &PvmContext, version: &str, type_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(v) = ctx.db.get_php_version(version)? {
        println!("Version {} already exists", v.version);
        return Ok(());
    }

    // 1. Scrape php releases from remote site
    let scraped_urls = crate::helpers::scrape_php_releases()?;
    ctx.db.clear_install_urls()?;
    for u in scraped_urls {
        ctx.db.add_install_url(&u)?;
    }

    // 2. Select system architecture
    let arch = if cfg!(target_arch = "x86_64") { "x64" } else { "x86" };

    // 3. Select matching InstallUrl
    let install_url = match ctx.db.get_install_url(version, arch, type_str)? {
        Some(u) => u,
        None => {
            println!("Download url not found for version {}", version);
            return Ok(());
        }
    };

    let version_install = install_url.version;
    let path = ctx.base_dir.join(format!("php{}", version_install));
    let zip_path = ctx.base_dir.join(format!("php{}.zip", version_install));

    if zip_path.exists() {
        fs::remove_file(&zip_path)?;
    }

    // 4. Download file
    crate::helpers::download_file_with_progress(&install_url.url, &zip_path)?;

    // 5. Extract Zip
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    crate::helpers::extract_zip(&zip_path, &path)?;

    // 6. Update ini file
    crate::helpers::update_ini_file(&path)?;

    // 7. Cleanup ZIP file
    if zip_path.exists() {
        let _ = fs::remove_file(&zip_path);
    }

    // 8. Update DB
    ctx.db.remove_php_versions_by_name(&version_install)?;
    ctx.db.add_php_version(&version_install, &path.to_string_lossy(), false)?;

    println!("installed successfully {}", version_install);
    Ok(())
}
