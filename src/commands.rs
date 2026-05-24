use crate::db::Db;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PvmContext {
    pub base_dir: PathBuf,
    pub db: Db,
}

/// Add local php version to PVM.
pub fn add_command(
    ctx: &PvmContext,
    version: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
    ctx.db
        .add_php_version(version, &moved_path.to_string_lossy(), false)?;
    println!("Added version {}", version);
    Ok(())
}

/// List all installed php versions.
pub fn list_command(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Current working directory: {}",
        ctx.base_dir.to_string_lossy()
    );
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

    //  Move currently active php version back to php<version>
    if let Some(current_version) = ctx.db.get_current_php_version()? {
        let current_path = Path::new(&current_version.path);
        let moved_path = ctx.base_dir.join(format!("php{}", current_version.version));
        println!(
            "Moving {} to {}",
            current_version.path,
            moved_path.to_string_lossy()
        );
        if current_path.exists() {
            fs::rename(current_path, &moved_path)?;
        }
        println!(
            "Moved {} to {}",
            current_version.path,
            moved_path.to_string_lossy()
        );
        ctx.db.update_php_version_path_and_current(
            current_version.id,
            &moved_path.to_string_lossy(),
            false,
        )?;

        return setup_command(ctx);
    }

    // 2. Move target version directory to active php directory
    let active_php_dir = ctx.base_dir.join("php");
    fs::rename(target_path, &active_php_dir)?;
    ctx.db.update_php_version_path_and_current(
        php_version.id,
        &active_php_dir.to_string_lossy(),
        true,
    )?;
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
        println!(
            "Opening notepad is only supported on Windows. File path: {}",
            ini_path.to_string_lossy()
        );
    }
    Ok(())
}

/// Open extension folder in file explorer.
pub fn ext_command(
    ctx: &PvmContext,
    version: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
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
        println!(
            "Opening explorer is only supported on Windows. Directory path: {}",
            ext_path.to_string_lossy()
        );
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
pub fn install_command(
    ctx: &PvmContext,
    version: &str,
    type_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(v) = ctx.db.get_php_version(version)? {
        println!("Version {} already exists", v.version);
        return Ok(());
    }

    // Scrape php releases from remote site
    let scraped_urls = crate::helpers::scrape_php_releases()?;
    ctx.db.clear_install_urls()?;
    for u in scraped_urls {
        ctx.db.add_install_url(&u)?;
    }

    // Select system architecture
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "x86"
    };

    // Select matching InstallUrl
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

    // Download file
    crate::helpers::download_file_with_progress(&install_url.url, &zip_path)?;

    // 5. Extract Zip
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    crate::helpers::extract_zip(&zip_path, &path)?;

    // Update ini file
    crate::helpers::update_ini_file(&path)?;

    // Cleanup ZIP file
    if zip_path.exists() {
        let _ = fs::remove_file(&zip_path);
    }

    // Update DB
    ctx.db.remove_php_versions_by_name(&version_install)?;
    ctx.db
        .add_php_version(&version_install, &path.to_string_lossy(), false)?;

    println!("installed successfully {}", version_install);
    Ok(())
}

/// Uninstall/remove a registered PHP version.
pub fn uninstall_command(
    ctx: &PvmContext,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let php_version = ctx.db.get_php_version(version)?;
    let php_version = match php_version {
        Some(v) => v,
        None => {
            println!("Version {} not found", version);
            return Ok(());
        }
    };

    if php_version.is_current {
        println!(
            "Version {} is currently in use. Switch to another version before uninstalling.",
            php_version.version
        );
        return Ok(());
    }

    let path = Path::new(&php_version.path);
    if path.exists() {
        fs::remove_dir_all(path)?;
        println!("Deleted folder: {}", php_version.path);
    } else {
        println!("Folder not found: {}", php_version.path);
    }

    ctx.db.remove_php_versions_by_name(&php_version.version)?;
    println!("Uninstalled version {}", php_version.version);
    Ok(())
}

/// List all available remote PHP versions from windows.php.net.
pub fn list_remote_command(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching available remote versions from windows.php.net...");
    let scraped_urls = crate::helpers::scrape_php_releases()?;
    ctx.db.clear_install_urls()?;
    for u in scraped_urls {
        ctx.db.add_install_url(&u)?;
    }

    let urls = ctx.db.get_install_urls()?;
    if urls.is_empty() {
        println!("No remote versions found");
        return Ok(());
    }

    println!("\nAvailable remote versions:");
    println!("{:<12} | {:<5} | {:<5}", "Version", "Type", "Arch");
    println!("-------------------------------");
    for u in urls {
        println!("{:<12} | {:<5} | {:<5}", u.version, u.type_, u.architecture);
    }
    Ok(())
}

/// Set up PVM system environment path and import existing PHP version if found.
pub fn setup_command(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    run_elevated_if_needed()?;

    // PHP Detection
    let mut old_php_dir = None;
    let mut old_php_version = None;

    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            let php_exe = path.join("php.exe");
            if php_exe.exists() {
                // Skip if it is inside PVM's own directory
                if path.starts_with(&ctx.base_dir) {
                    continue;
                }

                if let Ok(version) = get_php_version_from_cli(&php_exe) {
                    old_php_dir = Some(path.clone());
                    old_php_version = Some(version);
                    break;
                }
            }
        }
    }

    if let (Some(dir), Some(version)) = (&old_php_dir, &old_php_version) {
        println!(
            "Detected existing PHP version {} installed at: {}",
            version,
            dir.to_string_lossy()
        );

        // Register it under PVM if not already registered
        if ctx.db.get_php_version_exact(version)?.is_none() {
            let moved_path = ctx.base_dir.join(format!("php{}", version));
            if !moved_path.exists() {
                println!("Importing existing PHP into PVM local store...");
                crate::helpers::copy_dir_all(dir, &moved_path)?;
                ctx.db
                    .add_php_version(version, &moved_path.to_string_lossy(), false)?;
                println!("Imported version {} successfully.", version);
            }
        } else {
            println!("Version {} is already registered under PVM.", version);
        }
    }

    // Registry PATH modification
    #[cfg(target_os = "windows")]
    {
        update_system_path_windows(ctx, old_php_dir.as_deref())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("Updating System PATH is only supported on Windows.");
        if let Some(dir) = old_php_dir {
            println!(
                "You should manually remove '{}' from your PATH.",
                dir.to_string_lossy()
            );
        }
        println!(
            "You should manually add PVM directories to your PATH:\n  - {}\n  - {}",
            ctx.base_dir.to_string_lossy(),
            ctx.base_dir.join("php").to_string_lossy()
        );
    }

    // Pause if we were auto-elevated so the user can read the console output
    let is_auto_elevated = std::env::args().any(|a| a == "--pvm-auto-elevated");
    if is_auto_elevated {
        println!("\nPress Enter to close this window...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }

    Ok(())
}

fn get_php_version_from_cli(php_exe_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new(php_exe_path)
        .arg("-v")
        .output()?;
    if !output.status.success() {
        return Err("Failed to run php -v".into());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().ok_or("Empty php -v output")?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() > 1 && parts[0] == "PHP" {
        Ok(parts[1].to_string())
    } else {
        Err("Unexpected php -v output format".into())
    }
}

#[cfg(target_os = "windows")]
fn update_system_path_windows(
    ctx: &PvmContext,
    remove_dir: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env_key_path = r"System\CurrentControlSet\Control\Session Manager\Environment";

    let env_key: RegKey = match hklm.open_subkey_with_flags(env_key_path, KEY_READ | KEY_WRITE) {
        Ok(key) => key,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err("Error: Administrator privileges are required to modify System Environment Variables.\nPlease restart your terminal as Administrator and try again.".into());
            } else {
                return Err(Box::new(e));
            }
        }
    };

    let raw_val = env_key.get_raw_value("Path")?;
    let path_type = raw_val.vtype;
    let current_path_str: String = env_key.get_value("Path")?;

    let pvm_dir = &ctx.base_dir;
    let pvm_php_dir = ctx.base_dir.join("php");

    let new_path_str = crate::helpers::clean_and_update_path_string(
        &current_path_str,
        remove_dir,
        pvm_dir,
        &pvm_php_dir,
    )?;

    let mut bytes: Vec<u8> = new_path_str
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    bytes.push(0);
    bytes.push(0);

    let reg_value = winreg::RegValue {
        vtype: path_type,
        bytes,
    };
    env_key.set_raw_value("Path", &reg_value)?;

    broadcast_settings_change();

    println!("Successfully updated System PATH environment variable.");
    println!(
        "Paths added:\n  - {}\n  - {}",
        pvm_dir.to_string_lossy(),
        pvm_php_dir.to_string_lossy()
    );
    Ok(())
}

#[cfg(target_os = "windows")]
fn broadcast_settings_change() {
    unsafe extern "system" {
        fn SendMessageTimeoutW(
            hWnd: *mut std::ffi::c_void,
            Msg: u32,
            wParam: usize,
            lParam: *const u16,
            fuFlags: u32,
            uTimeout: u32,
            lpdwResult: *mut usize,
        ) -> isize;
    }

    let subkey = "Environment\0".encode_utf16().collect::<Vec<u16>>();
    const HWND_BROADCAST: *mut std::ffi::c_void = 0xffff as *mut std::ffi::c_void;
    const WM_SETTINGCHANGE: u32 = 0x001A;
    const SMTO_ABORTIFHUNG: u32 = 0x0002;

    let mut result = 0;
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            subkey.as_ptr(),
            SMTO_ABORTIFHUNG,
            5000,
            &mut result,
        );
    }
}

#[cfg(target_os = "windows")]
pub fn run_elevated_if_needed() -> Result<bool, Box<dyn std::error::Error>> {
    if is_elevated() {
        return Ok(false);
    }

    let exe_path = std::env::current_exe()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut args_str = args.join(" ");
    args_str.push_str(" --pvm-auto-elevated");

    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    let file_w: Vec<u16> = exe_path.as_os_str().encode_wide().chain(Some(0)).collect();
    let parameters_w: Vec<u16> = std::ffi::OsString::from(args_str)
        .encode_wide()
        .chain(Some(0))
        .collect();
    let verb_w: Vec<u16> = std::ffi::OsString::from("runas")
        .encode_wide()
        .chain(Some(0))
        .collect();

    #[repr(C)]
    struct SHELLEXECUTEINFOW {
        cbSize: u32,
        fMask: u32,
        hwnd: *mut std::ffi::c_void,
        lpVerb: *const u16,
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShow: i32,
        hInstApp: *mut std::ffi::c_void,
        lpIDList: *mut std::ffi::c_void,
        lpClass: *const u16,
        hkeyClass: *mut std::ffi::c_void,
        dwHotKey: u32,
        hIconOrMonitor: *mut std::ffi::c_void,
        hProcess: *mut std::ffi::c_void,
    }

    unsafe extern "system" {
        fn ShellExecuteExW(pExecInfo: *mut SHELLEXECUTEINFOW) -> i32;
        fn WaitForSingleObject(hHandle: *mut std::ffi::c_void, dwMilliseconds: u32) -> u32;
        fn GetExitCodeProcess(hProcess: *mut std::ffi::c_void, lpExitCode: *mut u32) -> i32;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    }

    const SEE_MASK_NOCLOSEPROCESS: u32 = 0x00000040;
    const INFINITE: u32 = 0xFFFFFFFF;
    const SW_SHOWNORMAL: i32 = 1;

    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: ptr::null_mut(),
        lpVerb: verb_w.as_ptr(),
        lpFile: file_w.as_ptr(),
        lpParameters: parameters_w.as_ptr(),
        lpDirectory: ptr::null(),
        nShow: SW_SHOWNORMAL,
        hInstApp: ptr::null_mut(),
        lpIDList: ptr::null_mut(),
        lpClass: ptr::null(),
        hkeyClass: ptr::null_mut(),
        dwHotKey: 0,
        hIconOrMonitor: ptr::null_mut(),
        hProcess: ptr::null_mut(),
    };

    println!("Requesting Administrator elevation for setup...");
    let success = unsafe { ShellExecuteExW(&mut info) };
    if success == 0 {
        return Err("Elevation request rejected or failed.".into());
    }

    if !info.hProcess.is_null() {
        unsafe {
            WaitForSingleObject(info.hProcess, INFINITE);
            let mut exit_code: u32 = 0;
            GetExitCodeProcess(info.hProcess, &mut exit_code);
            CloseHandle(info.hProcess);
            std::process::exit(exit_code as i32);
        }
    }

    std::process::exit(0);
}

#[cfg(target_os = "windows")]
fn is_elevated() -> bool {
    unsafe extern "system" {
        fn OpenProcessToken(
            ProcessHandle: *mut std::ffi::c_void,
            DesiredAccess: u32,
            TokenHandle: *mut *mut std::ffi::c_void,
        ) -> i32;
        fn GetTokenInformation(
            TokenHandle: *mut std::ffi::c_void,
            TokenInformationClass: u32,
            TokenInformation: *mut std::ffi::c_void,
            TokenInformationLength: u32,
            ReturnLength: *mut u32,
        ) -> i32;
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    }

    const TOKEN_QUERY: u32 = 0x0008;
    const TOKEN_ELEVATION: u32 = 20;

    let mut token: *mut std::ffi::c_void = std::ptr::null_mut();
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) != 0 {
            let mut elevation: u32 = 0;
            let mut size: u32 = 0;
            let res = GetTokenInformation(
                token,
                TOKEN_ELEVATION,
                &mut elevation as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
                &mut size,
            );
            CloseHandle(token);
            if res != 0 {
                return elevation != 0;
            }
        }
    }
    false
}
