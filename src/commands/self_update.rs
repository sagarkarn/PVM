use crate::commands::PvmContext;
use std::fs;

/// Compare version strings. Returns true if latest is newer than current.
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let clean_latest = latest.trim_start_matches('v');
    let clean_current = current.trim_start_matches('v');
    let current_parts: Vec<u32> = clean_current.split('.').filter_map(|s| s.parse().ok()).collect();
    let latest_parts: Vec<u32> = clean_latest.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let curr = current_parts.get(i).cloned().unwrap_or(0);
        let lat = latest_parts.get(i).cloned().unwrap_or(0);
        if lat > curr {
            return true;
        } else if curr > lat {
            return false;
        }
    }
    false
}

/// Checks GitHub for any new release.
pub fn check_for_update(_ctx: &PvmContext) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) PVM-Updater")
        .build()?;
    let response = client.get("https://api.github.com/repos/sagarkarn/PVM/releases/latest").send()?;
    if !response.status().is_success() {
        return Err(format!("Failed to query GitHub Releases API: {}", response.status()).into());
    }
    let body = response.text()?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    let tag_name = json.get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or("Failed to parse tag_name from GitHub API response")?;

    let current_ver = crate::commands::PVM_VERSION;
    if is_newer_version(current_ver, tag_name) {
        let assets = json.get("assets")
            .and_then(|v| v.as_array())
            .ok_or("Failed to parse assets from GitHub API")?;

        let mut download_url = None;
        for asset in assets {
            if let Some(name) = asset.get("name").and_then(|v| v.as_str()) {
                if name.contains("windows-x64.zip") || name.contains("win-x64.zip") {
                    if let Some(url) = asset.get("browser_download_url").and_then(|v| v.as_str()) {
                        download_url = Some(url.to_string());
                        break;
                    }
                }
            }
        }
        if let Some(url) = download_url {
            return Ok(Some((tag_name.to_string(), url)));
        }
    }
    Ok(None)
}

/// Perform automatic daily check for updates and output notice if available.
pub fn auto_update_check(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    // Skip if running inside test mode
    if std::env::var("PVM_TEST_MODE").is_ok() && std::env::var("PVM_ALLOW_UPDATE_TEST").is_err() {
        return Ok(());
    }

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let last_check = ctx.db.get_setting("LastUpdateCheck")?;
    let should_check = match last_check {
        Some(val) => {
            if let Ok(last_secs) = val.parse::<u64>() {
                now_secs >= last_secs + 86400
            } else {
                true
            }
        }
        None => true,
    };

    if should_check {
        // Record check timestamp before executing request
        ctx.db.set_setting("LastUpdateCheck", &now_secs.to_string())?;

        // Perform the check (catch error so the main command continues running!)
        if let Ok(Some((tag_name, _))) = check_for_update(ctx) {
            println!("\n[Notice] A new version of PVM is available: {} (current: v{}).", tag_name, crate::commands::PVM_VERSION);
            println!("Run 'pvm self-update' to update automatically.\n");
        }
    }

    Ok(())
}

/// Run PVM self-update.
pub fn self_update_command(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking for updates on GitHub...");
    let update = check_for_update(ctx)?;
    let (tag_name, download_url) = match update {
        Some(val) => val,
        None => {
            println!("PVM is already up-to-date (v{}).", crate::commands::PVM_VERSION);
            return Ok(());
        }
    };

    println!("New version found: {}. Downloading update...", tag_name);
    let zip_path = ctx.base_dir.join("pvm_update.zip");
    if zip_path.exists() {
        let _ = fs::remove_file(&zip_path);
    }

    // Download zip
    crate::helpers::download_file_with_progress(&download_url, &zip_path)?;

    // Extract zip
    let extract_temp = ctx.base_dir.join("pvm_update_temp");
    if extract_temp.exists() {
        let _ = fs::remove_dir_all(&extract_temp);
    }
    crate::helpers::extract_zip(&zip_path, &extract_temp)?;

    // Find extracted PVM.exe
    let mut new_exe_path = extract_temp.join("PVM.exe");
    if !new_exe_path.exists() {
        new_exe_path = extract_temp.join("pvm.exe");
    }
    if !new_exe_path.exists() {
        // Check for any file ending in .exe
        let mut found = false;
        if let Ok(entries) = fs::read_dir(&extract_temp) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("exe") {
                    new_exe_path = entry.path();
                    found = true;
                    break;
                }
            }
        }
        if !found {
            // Cleanup and exit
            let _ = fs::remove_file(&zip_path);
            let _ = fs::remove_dir_all(&extract_temp);
            return Err("Failed to find PVM.exe inside release zip".into());
        }
    }

    // Perform Windows-safe rename and hot-swap
    let current_exe = std::env::current_exe()?;
    let mut old_exe = current_exe.clone();
    old_exe.set_extension("exe.old");

    if old_exe.exists() {
        let _ = fs::remove_file(&old_exe);
    }

    // Rename current running exe to exe.old
    fs::rename(&current_exe, &old_exe)?;

    // Move new exe in place
    if let Err(e) = fs::rename(&new_exe_path, &current_exe) {
        // Rollback on failure
        let _ = fs::rename(&old_exe, &current_exe);
        let _ = fs::remove_file(&zip_path);
        let _ = fs::remove_dir_all(&extract_temp);
        return Err(format!("Failed to swap binaries: {}", e).into());
    }

    // Clean up temporary files
    let _ = fs::remove_file(&zip_path);
    let _ = fs::remove_dir_all(&extract_temp);

    println!("PVM successfully updated to {}!", tag_name);
    Ok(())
}
