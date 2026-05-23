use rusqlite::{params, Connection, Result};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PhpVersion {
    pub id: i32,
    pub version: String,
    pub path: String,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct InstallUrl {
    pub id: Option<i32>,
    pub version: String,
    pub url: String,
    pub type_: String,
    pub architecture: String,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Db { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS PhpVersions (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                IsCurrent INTEGER NOT NULL,
                Path TEXT NOT NULL,
                Version TEXT NOT NULL
            );",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS InstallUrls (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                Architecture TEXT NOT NULL,
                Type TEXT NOT NULL,
                Url TEXT NOT NULL,
                Version TEXT NOT NULL
            );",
            [],
        )?;

        Ok(())
    }

    pub fn get_php_versions(&self) -> Result<Vec<PhpVersion>> {
        let mut stmt = self.conn.prepare("SELECT Id, Version, Path, IsCurrent FROM PhpVersions")?;
        let rows = stmt.query_map([], |row| {
            Ok(PhpVersion {
                id: row.get(0)?,
                version: row.get(1)?,
                path: row.get(2)?,
                is_current: row.get::<_, i32>(3)? != 0,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_current_php_version(&self) -> Result<Option<PhpVersion>> {
        let mut stmt = self.conn.prepare("SELECT Id, Version, Path, IsCurrent FROM PhpVersions WHERE IsCurrent = 1 LIMIT 1")?;
        let mut rows = stmt.query_map([], |row| {
            Ok(PhpVersion {
                id: row.get(0)?,
                version: row.get(1)?,
                path: row.get(2)?,
                is_current: row.get::<_, i32>(3)? != 0,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn get_php_version(&self, version: &str) -> Result<Option<PhpVersion>> {
        // C# code: OrderByDescending(x=>x.Version).FirstOrDefault(v => v.Version.StartsWith(version))
        // So we order by Version DESC, and search for Version starting with the given version.
        let mut stmt = self.conn.prepare("SELECT Id, Version, Path, IsCurrent FROM PhpVersions WHERE Version LIKE ? ORDER BY Version DESC LIMIT 1")?;
        let pattern = format!("{}%", version);
        let mut rows = stmt.query_map([pattern], |row| {
            Ok(PhpVersion {
                id: row.get(0)?,
                version: row.get(1)?,
                path: row.get(2)?,
                is_current: row.get::<_, i32>(3)? != 0,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn get_php_version_exact(&self, version: &str) -> Result<Option<PhpVersion>> {
        let mut stmt = self.conn.prepare("SELECT Id, Version, Path, IsCurrent FROM PhpVersions WHERE Version = ? LIMIT 1")?;
        let mut rows = stmt.query_map([version], |row| {
            Ok(PhpVersion {
                id: row.get(0)?,
                version: row.get(1)?,
                path: row.get(2)?,
                is_current: row.get::<_, i32>(3)? != 0,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn add_php_version(&self, version: &str, path: &str, is_current: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO PhpVersions (Version, Path, IsCurrent) VALUES (?, ?, ?)",
            params![version, path, if is_current { 1 } else { 0 }],
        )?;
        Ok(())
    }

    pub fn update_php_version_path_and_current(&self, id: i32, path: &str, is_current: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE PhpVersions SET Path = ?, IsCurrent = ? WHERE Id = ?",
            params![path, if is_current { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn remove_php_versions_by_name(&self, version: &str) -> Result<()> {
        self.conn.execute("DELETE FROM PhpVersions WHERE Version = ?", [version])?;
        Ok(())
    }

    pub fn clear_install_urls(&self) -> Result<()> {
        self.conn.execute("DELETE FROM InstallUrls", [])?;
        Ok(())
    }

    pub fn add_install_url(&self, url: &InstallUrl) -> Result<()> {
        self.conn.execute(
            "INSERT INTO InstallUrls (Version, Url, Architecture, Type) VALUES (?, ?, ?, ?)",
            params![url.version, url.url, url.architecture, url.type_],
        )?;
        Ok(())
    }

    pub fn get_install_url(&self, version_prefix: &str, arch: &str, type_: &str) -> Result<Option<InstallUrl>> {
        // C# code: FirstOrDefault(u => u.Version.StartsWith(version) && u.Architecture == arch && u.Type == type)
        let mut stmt = self.conn.prepare("SELECT Id, Version, Url, Type, Architecture FROM InstallUrls WHERE Version LIKE ? AND Architecture = ? AND Type = ? LIMIT 1")?;
        let pattern = format!("{}%", version_prefix);
        let mut rows = stmt.query_map(params![pattern, arch, type_], |row| {
            Ok(InstallUrl {
                id: Some(row.get(0)?),
                version: row.get(1)?,
                url: row.get(2)?,
                type_: row.get(3)?,
                architecture: row.get(4)?,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn get_install_urls(&self) -> Result<Vec<InstallUrl>> {
        let mut stmt = self.conn.prepare("SELECT Id, Version, Url, Type, Architecture FROM InstallUrls ORDER BY Version DESC, Type, Architecture")?;
        let rows = stmt.query_map([], |row| {
            Ok(InstallUrl {
                id: Some(row.get(0)?),
                version: row.get(1)?,
                url: row.get(2)?,
                type_: row.get(3)?,
                architecture: row.get(4)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }
}
