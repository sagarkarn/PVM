use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

/// Recursively copies a directory from src to dst.
pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Downloads a URL to a file, updating progress percentages in stdout.
pub fn download_file_with_progress(url: &str, dest_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36")
        .build()?;
    let mut response = client.get(url).send()?;
    if !response.status().is_success() {
        return Err(format!("HTTP request failed with status {}", response.status()).into());
    }

    let total_size = response.content_length();
    let mut file = File::create(dest_path)?;
    let mut buffer = [0; 8192];
    let mut downloaded = 0;

    println!("Downloading... 0%");

    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read;

        if let Some(total) = total_size {
            let percentage = (downloaded as f64 / total as f64) * 100.0;
            // Print progress using a backspace/carriage return
            print!("\rdownloading... {:.2}%", percentage);
            io::stdout().flush()?;
        } else {
            print!("\rdownloading... {} bytes", downloaded);
            io::stdout().flush()?;
        }
    }
    println!(); // print newline to clear the progress output line
    Ok(())
}

/// Scrapes the PHP releases page on windows.php.net and returns list of InstallUrls.
pub fn scrape_php_releases() -> Result<Vec<crate::db::InstallUrl>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/42.0.2311.135 Safari/537.36 Edge/12.246")
        .build()?;
    let response = client.get("https://windows.php.net/downloads/releases/").send()?;
    if !response.status().is_success() {
        return Err(format!("Failed to retrieve PHP releases list: {}", response.status()).into());
    }

    let html = response.text()?;
    let document = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("a").unwrap();
    let mut install_urls = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            // Check filters: must contain "php-" and NOT contain debug, devel, test
            if href.contains("php-") && !(href.contains("debug") || href.contains("devel") || href.contains("test")) {
                let filename = href.split('/').last().unwrap_or("");
                let parts: Vec<&str> = filename.split('-').collect();
                if parts.len() > 1 {
                    let version = parts[1].to_string();
                    let arch = if filename.contains("x64") { "x64" } else { "x86" };
                    let type_ = if filename.contains("nts") { "nts" } else { "ts" };

                    let full_url = if href.starts_with('/') {
                        format!("https://windows.php.net{}", href)
                    } else if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://windows.php.net/downloads/releases/{}", href)
                    };

                    install_urls.push(crate::db::InstallUrl {
                        id: None,
                        version,
                        url: full_url,
                        architecture: arch.to_string(),
                        type_: type_.to_string(),
                    });
                }
            }
        }
    }

    Ok(install_urls)
}

/// Extracts a zip file to the target path.
pub fn extract_zip(zip_path: &Path, extract_to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    fs::create_dir_all(extract_to)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_to.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

/// Updates the php.ini file directory configuration and default curl extension.
pub fn update_ini_file(php_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let ini_path = php_path.join("php.ini");
    if !ini_path.exists() {
        let dev_path = php_path.join("php.ini-development");
        if dev_path.exists() {
            fs::copy(&dev_path, &ini_path)?;
        } else {
            return Err("Neither php.ini nor php.ini-development was found".into());
        }
    }

    let content = fs::read_to_string(&ini_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // 1. Configure extension_dir to "ext"
    if let Some(index) = lines.iter().position(|l| l.starts_with("extension_dir") || l.starts_with(";extension_dir")) {
        lines[index] = "extension_dir = \"ext\"".to_string();
    }

    // 2. Enable curl extension
    if let Some(index) = lines.iter().position(|l| l.starts_with(";extension=curl")) {
        lines[index] = "extension=curl".to_string();
    }

    fs::write(&ini_path, lines.join("\r\n"))?;
    Ok(())
}
