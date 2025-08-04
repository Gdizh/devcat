use crate::Result;
use clap::Parser;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(about = "Deletes the entire .devcat directory and all snapshots.")]
pub struct CleanArgs {
    #[arg(short, long, help = "Skip confirmation prompt.")]
    pub force: bool,
}

pub fn run(args: CleanArgs) -> Result<()> {
    let history_dir = std::path::PathBuf::from(".devcat");
    if !history_dir.exists() {
        println!("No .devcat directory found. Nothing to clean.");
        return Ok(());
    }
    
    if !args.force {
        println!("This will permanently delete all devcat snapshots for this project.");
        print!("Are you sure you want to continue? (y/N): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Clean operation cancelled.");
            return Ok(());
        }
    }

    std::fs::remove_dir_all(history_dir)?;
    println!("✅ .devcat directory and all snapshots have been deleted.");
    Ok(())
}
