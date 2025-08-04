use crate::{config, utils, Result, ExcludeArgs};
use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(about = "Saves a new versioned snapshot of the current code state.")]
pub struct SaveArgs {
    /// A short message describing the snapshot.
    #[arg(required = true)]
    pub message: String,
    #[command(flatten)]
    pub exclude_args: ExcludeArgs,
}

pub fn run(args: SaveArgs) -> Result<()> {
    let root_path = Path::new(".");
    let config = config::load_config(root_path)?;
    
    let mut excludes = args.exclude_args.exclude;
    excludes.extend(config.exclude);
    
    match utils::perform_save(root_path, &args.message, &excludes)? {
        utils::SaveStatus::Saved { id, message } => {
            println!("✅ Snapshot {} saved: {}", id, message);
        }
        utils::SaveStatus::NoChanges => {
            println!("✅ No changes detected since last snapshot. Nothing to save.");
        }
    }
    Ok(())
}
