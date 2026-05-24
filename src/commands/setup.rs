use crate::commands::PvmContext;
use std::path::Path;

/// Set up PVM system environment path and import existing PHP version if found.
pub fn setup_command(ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("PVM_TEST_MODE").is_ok() {
        return Ok(());
    }

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
