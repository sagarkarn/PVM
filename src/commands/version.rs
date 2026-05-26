use crate::commands::PvmContext;

/// The central version of PVM.
pub const PVM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Print the current PVM version.
pub fn version_command(_ctx: &PvmContext) -> Result<(), Box<dyn std::error::Error>> {
    println!("PVM version v{}", PVM_VERSION);
    Ok(())
}
