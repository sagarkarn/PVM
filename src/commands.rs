pub mod add;
pub mod list;
pub mod use_cmd;
pub mod ini;
pub mod ext;
pub mod ext_enable;
pub mod install;
pub mod uninstall;
pub mod list_remote;
pub mod setup;
pub mod self_update;

use crate::db::Db;
use std::path::PathBuf;

pub struct PvmContext {
    pub base_dir: PathBuf,
    pub db: Db,
}

pub use add::add_command;
pub use list::list_command;
pub use use_cmd::use_command;
pub use ini::ini_command;
pub use ext::ext_command;
pub use ext_enable::ext_enable_command;
pub use install::install_command;
pub use uninstall::uninstall_command;
pub use list_remote::list_remote_command;
pub use setup::setup_command;
pub use self_update::{self_update_command, auto_update_check, is_newer_version};
