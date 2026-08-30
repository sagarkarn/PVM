use anyhow::{Context, Result, anyhow};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    fs,
    process::{Command, Stdio},
};

use crate::commands::PvmContext;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub fn ext_install_command(ctx: &PvmContext, ext: &str) -> Result<()> {
    let bin_dir = ctx.base_dir.join("bin");
    let pie_phar_path = bin_dir.join("pie.phar");

    if !pie_phar_path.exists() {
        println!("PIE not found. Downloading latest version...");

        download_pie(&bin_dir, &pie_phar_path)?;
    }

    println!("Installing PHP extension: {ext}");

    let status = Command::new("php")
        .arg(&pie_phar_path)
        .arg("install")
        .arg(ext)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to start PHP/PIE")?;

    if !status.success() {
        return Err(anyhow!(
            "PIE failed to install extension `{}` (exit code: {:?})",
            ext,
            status.code()
        ));
    }

    println!("Extension `{ext}` installed successfully.");

    Ok(())
}

fn download_pie(bin_dir: &std::path::Path, pie_phar_path: &std::path::Path) -> Result<()> {
    const RELEASE_URL: &str = "https://api.github.com/repos/php/pie/releases/latest";

    let client = Client::builder()
        .user_agent("PVM")
        .build()
        .context("Failed to create HTTP client")?;

    let release: GitHubRelease = client
        .get(RELEASE_URL)
        .send()
        .context("Failed to fetch latest PIE release")?
        .error_for_status()
        .context("GitHub returned an error while fetching PIE release")?
        .json()
        .context("Failed to parse GitHub release information")?;

    let download_url = release
        .assets
        .iter()
        .find(|asset| asset.name == "pie.phar")
        .map(|asset| asset.browser_download_url.as_str())
        .ok_or_else(|| anyhow!("Could not find `pie.phar` in the latest PIE release"))?;

    println!("Downloading PIE...");

    let bytes = client
        .get(download_url)
        .send()
        .context("Failed to download PIE")?
        .error_for_status()
        .context("Failed to download PIE: HTTP error")?
        .bytes()
        .context("Failed to read PIE download")?;

    fs::create_dir_all(bin_dir)
        .with_context(|| format!("Failed to create {}", bin_dir.display()))?;

    fs::write(pie_phar_path, bytes)
        .with_context(|| format!("Failed to write {}", pie_phar_path.display()))?;

    println!("PIE downloaded to {}", pie_phar_path.display());

    Ok(())
}
