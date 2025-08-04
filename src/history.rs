use crate::error::{Error, Result};
use crate::utils;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snapshot {
    pub id: u32,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub manifest_hash: String,
}

#[derive(Debug, Default, Clone)]
pub struct History {
    pub snapshots: Vec<Snapshot>,
    path: PathBuf,
}

impl History {
    pub fn load(_root_path: &Path) -> Result<Self> {
        let history_dir = utils::ensure_history_dir_exists()?;
        let path = history_dir.join("history.log");
        
        let mut snapshots: Vec<Snapshot> = Vec::new();
        if path.exists() {
            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if !line.trim().is_empty() {
                    snapshots.push(serde_json::from_str(&line)?);
                }
            }
        }
        snapshots.sort_by_key(|s| s.id);
        Ok(History { snapshots, path })
    }

    pub fn save(&self) -> Result<()> {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&self.path)?;
        for snapshot in &self.snapshots {
            let line = serde_json::to_string(snapshot)?;
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }

    pub fn add_snapshot(&mut self, message: String, manifest_hash: String) {
        let next_id = self.snapshots.last().map_or(1, |s| s.id + 1);
        self.snapshots.push(Snapshot {
            id: next_id,
            timestamp: Utc::now(),
            message,
            manifest_hash,
        });
    }
    
    pub fn get_snapshot(&self, id: u32) -> Result<&Snapshot> {
        self.snapshots.iter().find(|s| s.id == id).ok_or(Error::SnapshotIdNotFound(id))
    }

    pub fn get_latest(&self) -> Result<&Snapshot> {
        self.snapshots.last().ok_or(Error::NoSnapshots)
    }
}
