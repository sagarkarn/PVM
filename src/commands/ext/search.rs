use std::error::Error;

use reqwest::blocking::Client;

pub struct PackagistClient {
    client: Client,
}

pub struct SearchResult {
    pub extensions: Vec<Extension>,
    pub total: u64,
}

#[derive(Debug)]
pub struct Extension {
    pub name: String,
    pub description: String,
    pub url: String,
    pub repository: String,
    pub downloads: u64,
    pub abandoned: bool,
}

impl PackagistClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn search_extension(&self, query: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("https://packagist.org/search.json?q={}&type=php-ext", query);

        let response = self.client.get(&url).send()?;
        let body = response.text()?;
        let json: serde_json::Value = serde_json::from_str(&body)?;

        let total = json["total"].as_u64().unwrap_or(0);
        let results = json["results"].as_array().unwrap();
        let mut extensions = Vec::new();
        for result in results {
            let name = result["name"].as_str().unwrap_or("");
            let description = result["description"].as_str().unwrap_or("");
            let url = result["url"].as_str().unwrap_or("");
            let repository = result["repository"].as_str().unwrap_or("");
            let downloads = result["downloads"].as_u64().unwrap_or(0);
            let abandoned = result["abandoned"].as_bool().unwrap_or(false);
            extensions.push(Extension {
                name: name.to_string(),
                description: description.to_string(),
                url: url.to_string(),
                repository: repository.to_string(),
                downloads,
                abandoned,
            });
        }

        for ext in &extensions {
            println!("{}", ext.name)
        }

        println!("Total {} extensions found", total);

        Ok(())
    }
}
