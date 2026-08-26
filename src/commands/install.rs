use anyhow::Result;

use crate::commands::PvmContext;
use std::fs;

/// Install PHP from windows.php.net.
pub fn install_command(ctx: &PvmContext, version: &str, type_str: &str) -> Result<()> {
    if let Some(v) = ctx.db.get_php_version(version).unwrap() {
        println!("Version {} already exists", v.version);
        return Ok(());
    }

    // Scrape php releases from remote site
    let scraped_urls = crate::helpers::scrape_php_releases().unwrap();
    ctx.db.clear_install_urls().unwrap();
    for u in scraped_urls {
        ctx.db.add_install_url(&u).unwrap();
    }

    // Select system architecture
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "x86"
    };

    // Select matching InstallUrl
    let install_url = match ctx.db.get_install_url(version, arch, type_str).unwrap() {
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
        fs::remove_file(&zip_path).unwrap();
    }

    // Download file
    crate::helpers::download_file_with_progress(&install_url.url, &zip_path).unwrap();

    // 5. Extract Zip
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    crate::helpers::extract_zip(&zip_path, &path).unwrap();

    // Update ini file
    crate::helpers::update_ini_file(&path).unwrap();

    // Cleanup ZIP file
    if zip_path.exists() {
        let _ = fs::remove_file(&zip_path);
    }

    // Update DB
    ctx.db
        .remove_php_versions_by_name(&version_install)
        .unwrap();
    ctx.db
        .add_php_version(&version_install, &path.to_string_lossy(), false)
        .unwrap();

    println!("installed successfully {}", version_install);
    Ok(())
}
