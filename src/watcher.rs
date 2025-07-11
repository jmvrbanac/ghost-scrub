use crate::config::GhostScrubConfig;
use crate::processor::{FileProcessor, ProcessResult};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct FileWatcher {
    processor: FileProcessor,
}

impl FileWatcher {
    pub fn new(config: GhostScrubConfig) -> Self {
        let processor = FileProcessor::new(config);
        Self { processor }
    }

    pub fn watch_paths(&self, paths: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if let Err(e) = tx.send(event) {
                            eprintln!("Error sending watch event: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Watch error: {}", e),
                }
            },
            Config::default(),
        )?;

        for path in paths {
            println!("Watching: {}", path.display());
            watcher.watch(path, RecursiveMode::Recursive)?;
        }

        println!("File watcher started. Press Ctrl+C to stop.");

        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    if let Err(e) = self.handle_event(event) {
                        eprintln!("Error handling event: {}", e);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Continue the loop
                }
                Err(e) => {
                    eprintln!("Watch receive error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn handle_event(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if path.is_file() && self.should_process_file(&path) {
                        match self.processor.process_file(&path, false, false) {
                            Ok(ProcessResult::Cleaned(count)) => {
                                println!("Auto-cleaned {} invisible characters from: {}", count, path.display());
                            }
                            Ok(ProcessResult::NoChanges) => {
                                // Silent for no changes in watch mode
                            }
                            Ok(ProcessResult::Skipped) => {
                                // Silent for skipped files
                            }
                            Ok(ProcessResult::DryRun(_)) => {
                                // This shouldn't happen in watch mode
                            }
                            Err(e) => {
                                eprintln!("Error processing {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
            _ => {
                // Ignore other event types (delete, etc.)
            }
        }
        Ok(())
    }

    fn should_process_file(&self, path: &Path) -> bool {
        // Skip temporary files, swap files, and hidden files commonly created by editors
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with('.') ||
               file_name.ends_with('~') ||
               file_name.ends_with(".tmp") ||
               file_name.ends_with(".swp") ||
               file_name.contains(".#") {
                return false;
            }
        }

        true
    }
}