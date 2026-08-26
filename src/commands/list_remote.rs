use anyhow::Result;

use crate::commands::PvmContext;

/// List all available remote PHP versions from windows.php.net.
pub fn list_remote_command(ctx: &PvmContext) -> Result<()> {
    println!("Fetching available remote versions from windows.php.net...");
    let scraped_urls = crate::helpers::scrape_php_releases().unwrap();
    ctx.db.clear_install_urls().unwrap();
    for u in scraped_urls {
        ctx.db.add_install_url(&u).unwrap();
    }

    let urls = ctx.db.get_install_urls().unwrap();
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
