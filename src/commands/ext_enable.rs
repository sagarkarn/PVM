use crate::commands::PvmContext;
use std::fs;

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
