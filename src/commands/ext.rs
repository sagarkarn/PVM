pub mod enable;
pub mod info;
pub mod install;
pub mod list;
pub mod open;
pub mod search;

use crate::commands::{PvmContext, ext::open::ext_open_command};
use clap::Subcommand;
use std::path::Path;

#[derive(Subcommand)]
pub enum ExtCommand {
    Open,
    List,
    Enable { ext: String },
    Search { query: String },
    Info { ext: String },
    Install { ext: String },
}

pub fn ext_command(
    ctx: &PvmContext,
    version: Option<String>,
    command: Option<ExtCommand>,
) -> anyhow::Result<()> {
    let php_version = match version {
        Some(ver) => ctx.db.get_php_version_exact(&ver).unwrap(),
        None => ctx.db.get_current_php_version().unwrap(),
    };

    let command = match command {
        Some(cmd) => cmd,
        None => ExtCommand::Open,
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

    match command {
        ExtCommand::Open => ext_open_command(&ext_path),
        ExtCommand::List => list::list_ext_command(&ext_path, &php_version),
        ExtCommand::Enable { ext } => enable::ext_enable_command(ctx, &ext),
        ExtCommand::Search { query } => search::PackagistClient::new().search_extension(&query),
        ExtCommand::Info { ext } => info::ext_info_command(&ext_path, &ext),
        ExtCommand::Install { ext } => install::ext_install_command(ctx, &ext),
    }
    .unwrap();
    Ok(())
}
