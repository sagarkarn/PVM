pub mod add;
pub mod ext;
pub mod ini;
pub mod install;
pub mod list;
pub mod list_remote;
pub mod self_update;
pub mod setup;
pub mod uninstall;
pub mod use_cmd;
pub mod version;

use crate::db::Db;
use std::path::PathBuf;

pub struct PvmContext {
    pub base_dir: PathBuf,
    pub db: Db,
}

pub use add::add_command;
pub use ext::ext_command;
pub use ini::ini_command;
pub use install::install_command;
pub use list::list_command;
pub use list_remote::list_remote_command;
pub use self_update::{auto_update_check, is_newer_version, self_update_command};
pub use setup::setup_command;
pub use uninstall::uninstall_command;
pub use use_cmd::use_command;
pub use version::{PVM_VERSION, version_command};
