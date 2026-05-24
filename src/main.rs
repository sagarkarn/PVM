#![allow(non_snake_case)]

use clap::{Parser, Subcommand, ValueEnum};
use PVM::db::Db;
use PVM::commands::{
    add_command, ext_command, ext_enable_command, ini_command, install_command, list_command,
    list_remote_command, setup_command, uninstall_command, use_command, PvmContext,
};

#[derive(Parser)]
#[command(name = "pvm")]
#[command(about = "PHP Version Manager for Windows", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Let manage pvm add already installed php.
    Add {
        /// PHP version
        #[arg(long)]
        version: String,
        /// Path of local PHP installation
        #[arg(long)]
        path: String,
    },
    /// List all installed php version with its paths
    List,
    /// Switch to use the specified version
    Use {
        /// PHP Version to use (e.g. 8.3)
        version: String,
    },
    /// Open php.ini file in notepad
    Ini,
    /// Open extension folder in file explorer of current php version.
    Ext {
        /// Optional PHP Version to view extensions folder
        version: Option<String>,
    },
    /// Enable extension that is already installed in current php version
    #[command(name = "ext-enable")]
    ExtEnable {
        /// Name of extension to enable (e.g. curl)
        ext: String,
    },
    /// To download and install specific version on the system.
    Install {
        /// Version to install (e.g. 8.3.3)
        version: String,
        /// Only nts and ts value is allowed
        #[arg(long, default_value = "nts")]
        type_: PhpType,
    },
    /// Uninstall/remove a registered PHP version.
    Uninstall {
        /// PHP Version to uninstall
        version: String,
    },
    /// List all available remote PHP versions from windows.php.net.
    #[command(name = "list-remote")]
    ListRemote,
    /// Set up PVM path configurations and import existing PHP installations (requires Administrator).
    Setup,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum PhpType {
    Nts,
    Ts,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let base_dir = exe_path
        .parent()
        .ok_or("Failed to get executable directory")?
        .to_path_buf();

    let db_path = base_dir.join("pvm.db");
    let db = Db::new(&db_path)?;
    let ctx = PvmContext { base_dir, db };

    let args: Vec<String> = std::env::args()
        .filter(|a| a != "--pvm-auto-elevated")
        .collect();
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Add { version, path } => {
            add_command(&ctx, &version, &path)?;
        }
        Commands::List => {
            list_command(&ctx)?;
        }
        Commands::Use { version } => {
            use_command(&ctx, &version)?;
        }
        Commands::Ini => {
            ini_command(&ctx)?;
        }
        Commands::Ext { version } => {
            ext_command(&ctx, version)?;
        }
        Commands::ExtEnable { ext } => {
            ext_enable_command(&ctx, &ext)?;
        }
        Commands::Install { version, type_ } => {
            let type_str = match type_ {
                PhpType::Nts => "nts",
                PhpType::Ts => "ts",
            };
            install_command(&ctx, &version, type_str)?;
        }
        Commands::Uninstall { version } => {
            uninstall_command(&ctx, &version)?;
        }
        Commands::ListRemote => {
            list_remote_command(&ctx)?;
        }
        Commands::Setup => {
            setup_command(&ctx)?;
        }
    }

    Ok(())
}
