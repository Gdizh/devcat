use crate::{config, utils, Result};
use clap::Parser;
use log::{debug, info};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(about = "Automatically create snapshots when files change.")]
pub struct WatchArgs {}

pub fn run(_args: WatchArgs) -> Result<()> {
    let root_path = Path::new(".");
    let config = config::load_config(root_path)?;

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: notify::Result<Event>| {
            debug!("Received raw watch event: {:?}", res);
            if let Ok(event) = res {
                // This `matches!` block explicitly checks for the event kinds you mentioned.
                let should_trigger = matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                );

                if should_trigger {
                    if let Some(path) = event.paths.first() {
                        if !path.starts_with("./.devcat") {
                            // Send the specific event kind for better logging.
                            let _ = tx.send(event.kind);
                        }
                    }
                }
            }
        })?;

    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    println!("👀 Watching for file changes. Press Ctrl+C to exit.");

    let debounce_duration = Duration::from_secs(2);
    let mut last_event_time: Option<Instant> = None;

    loop {
        if let Some(kind) = rx.try_iter().last() {
            info!("Change detected ({:?}). Starting debounce timer...", kind);
            last_event_time = Some(Instant::now());
        }

        if let Some(last_event) = last_event_time {
            if last_event.elapsed() >= debounce_duration {
                let now = chrono::Local::now();
                let message = format!("Auto-snapshot @ {}", now.format("%Y-%m-%d %H:%M:%S"));
                
                match utils::perform_save(root_path, &message, &config.exclude) {
                    Ok(utils::SaveStatus::Saved { id, message }) => {
                        println!("\n-- Quiet period ended, snapshot {} created: \"{}\" --", id, message);
                    }
                    Ok(utils::SaveStatus::NoChanges) => {
                        // Silently do nothing if the content hasn't actually changed.
                    }
                    Err(e) => {
                        eprintln!("\nFailed to create auto-snapshot: {}", e);
                    }
                }
                last_event_time = None;
            }
        }
        
        std::thread::sleep(Duration::from_millis(500));
    }
}