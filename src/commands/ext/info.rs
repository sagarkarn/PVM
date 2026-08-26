use anyhow::{Context, Result, anyhow};
use goblin::pe::PE;
use pelite::PeFile;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn ext_info_command(ext_path: &Path, ext: &str) -> Result<()> {
    let file_name = format!("php_{ext}.dll");
    let full_path = ext_path.join(&file_name);

    if !full_path.exists() {
        return Err(anyhow!("PHP extension not found: {}", full_path.display()));
    }

    let buffer =
        fs::read(&full_path).with_context(|| format!("failed to read {}", full_path.display()))?;

    let pe = PE::parse(&buffer).map_err(|e| anyhow!("failed to parse PE file: {e}"))?;

    println!("Extension Info : {}", file_name);
    println!("----------------------------------------");

    // Architecture
    let arch = match pe.header.coff_header.machine {
        0x8664 => "x64 (64-bit)",
        0x014c => "x86 (32-bit)",
        0xaa64 => "ARM64",
        0x01c4 => "ARM",
        other => {
            return Err(anyhow!("unknown architecture: {:#x}", other));
        }
    };

    println!("Architecture   : {}", arch);

    // File size
    let metadata = fs::metadata(&full_path)
        .with_context(|| format!("failed to read metadata for {}", full_path.display()))?;

    println!(
        "File Size      : {:.2} MB",
        metadata.len() as f64 / 1_048_576.0
    );

    // DLL version
    match get_dll_version(&full_path) {
        Ok(version) => {
            println!("File Version   : {}", version);
        }
        Err(_) => {
            println!("File Version   : Not available");
        }
    }

    // Product version
    match get_product_version(&full_path) {
        Ok(version) => {
            println!("Product Version: {}", version);
        }
        Err(_) => {
            println!("Product Version: Not available");
        }
    }

    let extension_type = if pe
        .exports
        .iter()
        .any(|export| export.name == Some("get_module"))
    {
        "PHP Extension"
    } else if pe
        .exports
        .iter()
        .any(|export| export.name == Some("zend_extension_entry"))
    {
        "Zend Extension"
    } else {
        "Native DLL"
    };

    println!("Extension Type : {}", extension_type);

    // Exports
    println!();
    println!("Exports        :");

    for export in &pe.exports {
        if let Some(name) = export.name {
            println!("  -> {}", name);
        }
    }

    // Dependencies
    println!();
    println!("Dependencies   :");

    let mut dependencies = Vec::new();

    for import in &pe.imports {
        let dll_name = import.dll.to_ascii_lowercase();

        if !is_system_dll(&dll_name) && !dependencies.contains(&dll_name) {
            dependencies.push(dll_name);
        }
    }

    dependencies.sort();

    if dependencies.is_empty() {
        println!("  None");
    } else {
        for dependency in &dependencies {
            let dependency_path = find_dependency(dependency, &full_path, Some(ext_path));

            match dependency_path {
                Some(path) => {
                    println!("  ✓ {} -> {}", dependency, path.display());
                }
                None => {
                    println!("  ✗ {} -> MISSING", dependency);
                }
            }
        }
    }

    println!();
    println!("----------------------------------------");

    Ok(())
}

/// Get FileVersion from Windows VERSIONINFO resources.
fn get_dll_version(file_path: &Path) -> Result<String> {
    let bytes =
        fs::read(file_path).with_context(|| format!("failed to read {}", file_path.display()))?;

    let pe = PeFile::from_bytes(&bytes).map_err(|e| anyhow!("failed to parse PE: {e}"))?;

    let resources = pe
        .resources()
        .map_err(|e| anyhow!("version resources not found: {e}"))?;

    let version_info = resources
        .version_info()
        .map_err(|e| anyhow!("version info not found: {e}"))?;

    // Prefer the human-readable FileVersion string.
    for lang in version_info.translation() {
        if let Some(value) = version_info.value(*lang, "FileVersion") {
            let value = value.trim();

            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }

    // Fallback to VS_FIXEDFILEINFO.
    let fixed = version_info
        .fixed()
        .ok_or_else(|| anyhow!("fixed file version information not found"))?;

    let version = format!(
        "{}.{}.{}.{}",
        fixed.dwFileVersion.Major,
        fixed.dwFileVersion.Minor,
        fixed.dwFileVersion.Patch,
        fixed.dwFileVersion.Build,
    );

    Ok(version)
}

/// Get ProductVersion from Windows VERSIONINFO resources.
fn get_product_version(file_path: &Path) -> Result<String> {
    let bytes =
        fs::read(file_path).with_context(|| format!("failed to read {}", file_path.display()))?;

    let pe = PeFile::from_bytes(&bytes).map_err(|e| anyhow!("failed to parse PE: {e}"))?;

    let resources = pe
        .resources()
        .map_err(|e| anyhow!("resources not found: {e}"))?;

    let version_info = resources
        .version_info()
        .map_err(|e| anyhow!("version info not found: {e}"))?;

    for lang in version_info.translation() {
        if let Some(value) = version_info.value(*lang, "ProductVersion") {
            let value = value.trim();

            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }

    let fixed = version_info
        .fixed()
        .ok_or_else(|| anyhow!("fixed file version information not found"))?;

    let version = format!(
        "{}.{}.{}.{}",
        fixed.dwProductVersion.Major,
        fixed.dwProductVersion.Minor,
        fixed.dwProductVersion.Patch,
        fixed.dwProductVersion.Build,
    );

    Ok(version)
}

/// Determine whether a DLL is a Windows/system DLL.
///
/// We intentionally ignore these from the "interesting dependencies"
/// output because a PHP extension can have many of them.
fn is_system_dll(name: &str) -> bool {
    name.starts_with("api-ms-win")
        || name.starts_with("ext-ms-win")
        || matches!(
            name,
            "kernel32.dll"
                | "kernelbase.dll"
                | "user32.dll"
                | "advapi32.dll"
                | "gdi32.dll"
                | "gdi32full.dll"
                | "ntdll.dll"
                | "msvcrt.dll"
                | "ucrtbase.dll"
                | "ole32.dll"
                | "oleaut32.dll"
                | "shell32.dll"
                | "ws2_32.dll"
                | "bcrypt.dll"
                | "crypt32.dll"
        )
}

/// Search for a dependency DLL.
///
/// Search order:
///
/// 1. Extension directory
/// 2. PHP directory
/// 3. Windows System32
/// 4. Windows directory
/// 5. PATH
fn find_dependency(dependency: &str, dll_path: &Path, ext_path: Option<&Path>) -> Option<PathBuf> {
    let mut search_paths = Vec::new();

    // 1. Directory containing the DLL.
    if let Some(parent) = dll_path.parent() {
        search_paths.push(parent.to_path_buf());
    }

    // 2. Extension/PHP directory.
    if let Some(ext_path) = ext_path {
        search_paths.push(ext_path.to_path_buf());

        if let Some(php_dir) = ext_path.parent() {
            search_paths.push(php_dir.to_path_buf());
        }
    }

    // 3/4. Windows directories.
    if let Ok(windows_dir) = std::env::var("WINDIR") {
        let windows_dir = PathBuf::from(windows_dir);

        search_paths.push(windows_dir.join("System32"));
        search_paths.push(windows_dir);
    }

    // 5. PATH.
    if let Ok(path) = std::env::var("PATH") {
        search_paths.extend(std::env::split_paths(&path));
    }

    for directory in search_paths {
        let candidate = directory.join(dependency);

        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}
