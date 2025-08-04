use crate::{history::History, utils, Result};
use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(about = "Deletes old snapshots, keeping a specified number of recent ones.")]
pub struct PruneArgs {
    /// The number of recent snapshots to keep.
    #[arg(long, default_value_t = 10)]
    pub keep: usize,
}

pub fn run(args: PruneArgs) -> Result<()> {
    let root_path = Path::new(".");
    let mut history = History::load(root_path)?;
    
    if history.snapshots.len() <= args.keep {
        println!("Number of snapshots ({}) is less than or equal to the number to keep ({}). Nothing to prune.", history.snapshots.len(), args.keep);
        return Ok(());
    }

    let original_len = history.snapshots.len();
    let snapshots_to_keep = history.snapshots.split_off(original_len - args.keep);
    let snapshots_to_delete = history.snapshots;
    
    let mut kept_hashes = HashSet::new();
    for snapshot in &snapshots_to_keep {
        kept_hashes.insert(snapshot.manifest_hash.clone());
        if let Ok(manifest) = utils::get_manifest_from_hash(root_path, &snapshot.manifest_hash) {
            for hash in manifest.values() {
                kept_hashes.insert(hash.clone());
            }
        }
    }
    
    let objects_dir = root_path.join(".devcat").join("objects");
    if objects_dir.exists() {
        for snapshot in &snapshots_to_delete {
            if !kept_hashes.contains(&snapshot.manifest_hash) {
                let _ = fs::remove_file(objects_dir.join(&snapshot.manifest_hash));
            }
            if let Ok(manifest) = utils::get_manifest_from_hash(root_path, &snapshot.manifest_hash) {
                for hash in manifest.values() {
                    if !kept_hashes.contains(hash) {
                        let _ = fs::remove_file(objects_dir.join(hash));
                    }
                }
            }
        }
    }
    
    let mut final_history = History::load(root_path)?;
    final_history.snapshots = snapshots_to_keep;
    final_history.save()?;
    
    println!("✅ Pruned {} old snapshots. Kept {}.", snapshots_to_delete.len(), final_history.snapshots.len());
    Ok(())
}
