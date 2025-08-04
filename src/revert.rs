use crate::{history::History, Result};
use clap::Parser;
use ignore::WalkBuilder;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(about = "Reverts the working directory to a specific snapshot state.")]
pub struct RevertArgs {
    /// The snapshot ID to revert the working directory to.
    #[arg(required = true)]
    pub id: u32,
}

pub fn run(args: RevertArgs) -> Result<()> {
    let root_path = Path::new(".");
    let history = History::load(root_path)?;
    let snapshot = history.get_snapshot(args.id)?;

    let objects_dir = root_path.join(".devcat").join("objects");
    let manifest_content = fs::read(objects_dir.join(&snapshot.manifest_hash))?;
    let manifest: BTreeMap<PathBuf, String> = serde_json::from_slice(&manifest_content)?;

    for (path, hash) in &manifest {
        let content = fs::read(objects_dir.join(hash))?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, content)?;
    }

    let manifest_paths: HashSet<_> = manifest.keys().collect();
    let walker = WalkBuilder::new(root_path)
        .filter_entry(|entry| !entry.path().starts_with("./.devcat"))
        .build();

    for result in walker {
        let entry = result?;
        let path = entry.path();
        if path.is_file() {
            if let Ok(relative_path) = path.strip_prefix(root_path) {
                if !manifest_paths.contains(&relative_path.to_path_buf()) {
                    fs::remove_file(path)?;
                }
            }
        }
    }
    
    println!("✅ Reverted working directory to snapshot {}.", args.id);
    Ok(())
}
