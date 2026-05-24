use std::fs;
use std::path::PathBuf;
use PVM::db::Db;
use PVM::commands::{add_command, list_command, use_command, ext_enable_command, PvmContext};

fn setup_test_context(name: &str) -> (PathBuf, PvmContext) {
    let mut test_dir = std::env::current_dir().unwrap();
    test_dir.push("target");
    test_dir.push("test_runs");
    test_dir.push(name);

    if test_dir.exists() {
        let _ = fs::remove_dir_all(&test_dir);
    }
    fs::create_dir_all(&test_dir).unwrap();
    unsafe {
        std::env::set_var("PVM_TEST_MODE", "1");
    }

    let db_path = test_dir.join("pvm.db");
    let db = Db::new(&db_path).unwrap();

    (
        test_dir.clone(),
        PvmContext {
            base_dir: test_dir,
            db,
        },
    )
}

#[test]
fn test_add_command() {
    let (test_dir, ctx) = setup_test_context("test_add_command");

    // Create a mock local php installation directory
    let local_php_dir = test_dir.join("local_php_8.3.3");
    fs::create_dir_all(&local_php_dir).unwrap();
    fs::write(local_php_dir.join("php.exe"), "dummy php executable").unwrap();

    // Run add command
    add_command(&ctx, "8.3.3", &local_php_dir.to_string_lossy()).unwrap();

    // Verify DB
    let versions = ctx.db.get_php_versions().unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].version, "8.3.3");
    assert!(!versions[0].is_current);

    // Verify directory structure
    let copied_path = test_dir.join("php8.3.3");
    assert!(copied_path.exists());
    assert!(copied_path.join("php.exe").exists());
}

#[test]
fn test_list_command() {
    let (_test_dir, ctx) = setup_test_context("test_list_command");
    // Listing empty versions should succeed
    list_command(&ctx).unwrap();
}

#[test]
fn test_use_command() {
    let (test_dir, ctx) = setup_test_context("test_use_command");

    // Add version 8.3.3
    let local_php_1 = test_dir.join("local_php_1");
    fs::create_dir_all(&local_php_1).unwrap();
    fs::write(local_php_1.join("php.exe"), "php 8.3.3").unwrap();
    add_command(&ctx, "8.3.3", &local_php_1.to_string_lossy()).unwrap();

    // Add version 8.2.0
    let local_php_2 = test_dir.join("local_php_2");
    fs::create_dir_all(&local_php_2).unwrap();
    fs::write(local_php_2.join("php.exe"), "php 8.2.0").unwrap();
    add_command(&ctx, "8.2.0", &local_php_2.to_string_lossy()).unwrap();

    // Use 8.3.3
    use_command(&ctx, "8.3.3").unwrap();

    // The active php directory should contain php 8.3.3
    let active_php_dir = test_dir.join("php");
    assert!(active_php_dir.exists());
    assert_eq!(
        fs::read_to_string(active_php_dir.join("php.exe")).unwrap(),
        "php 8.3.3"
    );

    // Db status check
    let v833 = ctx.db.get_php_version_exact("8.3.3").unwrap().unwrap();
    assert!(v833.is_current);
    assert_eq!(v833.path, active_php_dir.to_string_lossy());

    // Use 8.2.0
    use_command(&ctx, "8.2.0").unwrap();

    // Active directory should now contain php 8.2.0
    assert_eq!(
        fs::read_to_string(active_php_dir.join("php.exe")).unwrap(),
        "php 8.2.0"
    );

    // Old version should have been moved back to php8.3.3 folder
    let moved_833_dir = test_dir.join("php8.3.3");
    assert!(moved_833_dir.exists());
    assert_eq!(
        fs::read_to_string(moved_833_dir.join("php.exe")).unwrap(),
        "php 8.3.3"
    );

    // Db status check
    let v833_updated = ctx.db.get_php_version_exact("8.3.3").unwrap().unwrap();
    assert!(!v833_updated.is_current);
    assert_eq!(v833_updated.path, moved_833_dir.to_string_lossy());

    let v820_updated = ctx.db.get_php_version_exact("8.2.0").unwrap().unwrap();
    assert!(v820_updated.is_current);
    assert_eq!(v820_updated.path, active_php_dir.to_string_lossy());
}

#[test]
fn test_ext_enable_command() {
    let (test_dir, ctx) = setup_test_context("test_ext_enable_command");

    // Create active php directory structure manually
    let active_php_dir = test_dir.join("php");
    let ext_dir = active_php_dir.join("ext");
    fs::create_dir_all(&ext_dir).unwrap();

    // Create a fake dll
    fs::write(ext_dir.join("php_curl.dll"), "fake dll").unwrap();

    // Create php.ini-development with commented extension
    let ini_content = ";extension=curl\r\n;extension_dir = \"ext\"\r\n;extension=mbstring\r\n";
    fs::write(active_php_dir.join("php.ini-development"), ini_content).unwrap();

    // Run ext_enable
    ext_enable_command(&ctx, "curl").unwrap();

    // php.ini should exist
    let ini_path = active_php_dir.join("php.ini");
    assert!(ini_path.exists());

    let content = fs::read_to_string(&ini_path).unwrap();
    assert!(content.contains("extension=curl"));
    assert!(!content.contains(";extension=curl"));
    assert!(content.contains(";extension=mbstring"));
}

#[test]
fn test_uninstall_command() {
    let (test_dir, ctx) = setup_test_context("test_uninstall_command");

    // Add version 8.3.3
    let local_php_1 = test_dir.join("local_php_1");
    fs::create_dir_all(&local_php_1).unwrap();
    fs::write(local_php_1.join("php.exe"), "php 8.3.3").unwrap();
    add_command(&ctx, "8.3.3", &local_php_1.to_string_lossy()).unwrap();

    // Add version 8.2.0
    let local_php_2 = test_dir.join("local_php_2");
    fs::create_dir_all(&local_php_2).unwrap();
    fs::write(local_php_2.join("php.exe"), "php 8.2.0").unwrap();
    add_command(&ctx, "8.2.0", &local_php_2.to_string_lossy()).unwrap();

    // Use 8.2.0 (so 8.2.0 is active/current, and 8.3.3 is inactive/uninstalled-eligible)
    use_command(&ctx, "8.2.0").unwrap();

    let inactive_dir = test_dir.join("php8.3.3");
    let active_dir = test_dir.join("php");
    assert!(inactive_dir.exists());
    assert!(active_dir.exists());

    // Try to uninstall current active version (should be refused/no-op)
    PVM::commands::uninstall_command(&ctx, "8.2.0").unwrap();
    assert!(active_dir.exists());
    assert!(ctx.db.get_php_version_exact("8.2.0").unwrap().is_some());

    // Uninstall inactive version
    PVM::commands::uninstall_command(&ctx, "8.3.3").unwrap();
    assert!(!inactive_dir.exists());
    assert!(ctx.db.get_php_version_exact("8.3.3").unwrap().is_none());
}

#[test]
fn test_list_remote_command() {
    let (_test_dir, ctx) = setup_test_context("test_list_remote_command");
    
    // Test that the database method get_install_urls works and starts empty
    let list = ctx.db.get_install_urls().unwrap();
    assert!(list.is_empty());
    
    // Add mock install urls
    ctx.db.add_install_url(&PVM::db::InstallUrl {
        id: None,
        version: "8.3.3".to_string(),
        url: "https://windows.php.net/downloads/releases/php-8.3.3-Win32-vs16-x64.zip".to_string(),
        type_: "nts".to_string(),
        architecture: "x64".to_string(),
    }).unwrap();
    
    let list_after = ctx.db.get_install_urls().unwrap();
    assert_eq!(list_after.len(), 1);
    assert_eq!(list_after[0].version, "8.3.3");
}

#[test]
fn test_clean_and_update_path_string() {
    use std::path::Path;
    use PVM::helpers::clean_and_update_path_string;

    let pvm_dir = Path::new(r"C:\Users\devsa\.pvm");
    let pvm_php_dir = Path::new(r"C:\Users\devsa\.pvm\php");
    let old_php_dir = Path::new(r"C:\php");

    // Case 1: Simple PATH containing old PHP and other system dirs.
    // It should remove old PHP, and append PVM and PVM/php.
    let current_path = r"C:\Windows;C:\Windows\System32;C:\php;C:\Program Files\Git\cmd";
    let new_path = clean_and_update_path_string(
        current_path,
        Some(old_php_dir),
        pvm_dir,
        pvm_php_dir,
    )
    .unwrap();

    // Verify it split/joined correctly with semicolon on Windows style paths
    let paths: Vec<String> = std::env::split_paths(&new_path)
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    
    assert!(!paths.contains(&r"C:\php".to_string()));
    assert!(paths.contains(&r"C:\Users\devsa\.pvm".to_string()));
    assert!(paths.contains(&r"C:\Users\devsa\.pvm\php".to_string()));
    assert!(paths.contains(&r"C:\Windows".to_string()));

    // Case 2: PVM paths already exist, and no old PHP directory to remove.
    // The PATH should contain both PVM paths.
    let current_path_2 = r"C:\Windows;C:\Users\devsa\.pvm;C:\Users\devsa\.pvm\php";
    let new_path_2 = clean_and_update_path_string(
        current_path_2,
        None,
        pvm_dir,
        pvm_php_dir,
    )
    .unwrap();
    
    let paths_2: Vec<String> = std::env::split_paths(&new_path_2)
        .map(|p| p.to_string_lossy().to_string())
        .collect();
        
    assert!(paths_2.contains(&r"C:\Users\devsa\.pvm".to_string()));
    assert!(paths_2.contains(&r"C:\Users\devsa\.pvm\php".to_string()));
}


