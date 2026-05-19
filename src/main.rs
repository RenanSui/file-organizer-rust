#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod file;

use notify::EventKindMask;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify_debouncer_full::RecommendedCache;
use notify_debouncer_full::new_debouncer_opt;
use std::time::Duration;

use config::Config::{self};
use file::File::{self};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Config::create_default_config()?;
    let config = Config::load_config()?;
    println!("{config:#?}");

    // setup debouncer with custom event filtering
    let (tx, rx) = std::sync::mpsc::channel();

    // Configure notify to exclude noisy access events (OPEN/CLOSE)
    // Use CORE mask: CREATE, REMOVE, MODIFY_DATA, MODIFY_META, MODIFY_NAME
    let notify_config = notify::Config::default()
        .with_event_kinds(EventKindMask::ALL_MODIFY | EventKindMask::CREATE);

    // Use new_debouncer_opt for full control over the watcher configuration
    let mut debouncer = new_debouncer_opt::<_, RecommendedWatcher, RecommendedCache>(
        Duration::from_secs(2), // debounce timeout
        None,                   // tick rate (None = auto)
        tx,
        RecommendedCache::new(),
        notify_config,
    )?;

    debouncer.watch(&config.directories.source_dir, RecursiveMode::NonRecursive)?;

    for result in rx {
        match result {
            Ok(events) => {
                for event in events {
                    for path in event.paths.clone() {
                        // Ignore directories
                        if path.is_dir() {
                            println!("File is dir: {path:#?}");
                            continue;
                        }

                        // get file metadata: name, extension
                        // creates all directories
                        // rename file if it is a duplicate, then move file
                        // handle possible errors
                        if let Err(e) = File::from(path.clone())
                            .extract_metadata()
                            .and_then(|f| f.ensure_destination(&config))
                            .and_then(|f| f.ensure_unique_filename().move_file())
                        {
                            println!("{path:?}: {}", e.to_string());
                        }
                    }
                }
            }
            Err(errors) => errors.iter().for_each(|error| eprintln!("{error:?}")),
        }
    }

    Ok(())
}
