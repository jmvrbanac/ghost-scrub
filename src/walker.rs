use crate::config::GhostScrubConfig;
use crate::processor::{FileProcessor, ProcessResult};
use glob::{glob, Pattern};
use std::path::{Path, PathBuf};
use std::fs;

pub struct FileWalker {
    processor: FileProcessor,
    config: GhostScrubConfig,
}

impl FileWalker {
    pub fn new(config: GhostScrubConfig) -> Self {
        let processor = FileProcessor::new(config.clone());
        Self { processor, config }
    }

    pub fn process_paths(&self, paths: &[PathBuf], dry_run: bool, verbose: bool) -> Result<WalkResult, Box<dyn std::error::Error>> {
        let mut result = WalkResult::default();

        for path in paths {
            if path.is_file() {
                self.process_single_file(path, dry_run, verbose, &mut result)?;
            } else if path.is_dir() {
                self.process_directory(path, dry_run, verbose, &mut result)?;
            } else {
                // Handle as glob pattern
                self.process_glob_pattern(&path.to_string_lossy(), dry_run, verbose, &mut result)?;
            }
        }

        Ok(result)
    }

    fn process_single_file(&self, file_path: &Path, dry_run: bool, verbose: bool, result: &mut WalkResult) -> Result<(), Box<dyn std::error::Error>> {
        match self.processor.process_file(file_path, dry_run, verbose) {
            Ok(ProcessResult::Cleaned(count)) => {
                result.files_processed += 1;
                result.total_changes += count;
            }
            Ok(ProcessResult::DryRun(count)) => {
                result.files_processed += 1;
                result.total_changes += count;
            }
            Ok(ProcessResult::NoChanges) => {
                result.files_processed += 1;
            }
            Ok(ProcessResult::Skipped) => {
                result.files_skipped += 1;
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", file_path.display(), e);
                result.errors += 1;
            }
        }
        Ok(())
    }

    fn process_directory(&self, dir_path: &Path, dry_run: bool, verbose: bool, result: &mut WalkResult) -> Result<(), Box<dyn std::error::Error>> {
        self.walk_directory_recursive(dir_path, dry_run, verbose, result)?;
        Ok(())
    }

    fn walk_directory_recursive(&self, dir_path: &Path, dry_run: bool, verbose: bool, result: &mut WalkResult) -> Result<(), Box<dyn std::error::Error>> {
        let entries = fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if self.should_skip_path(&path) {
                    continue;
                }
                self.walk_directory_recursive(&path, dry_run, verbose, result)?;
            } else if path.is_file() {
                if !self.should_skip_path(&path) {
                    self.process_single_file(&path, dry_run, verbose, result)?;
                }
            }
        }

        Ok(())
    }

    fn process_glob_pattern(&self, pattern: &str, dry_run: bool, verbose: bool, result: &mut WalkResult) -> Result<(), Box<dyn std::error::Error>> {
        for entry in glob(pattern)? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        self.process_single_file(&path, dry_run, verbose, result)?;
                    } else if path.is_dir() {
                        self.process_directory(&path, dry_run, verbose, result)?;
                    }
                }
                Err(e) => {
                    eprintln!("Glob error: {}", e);
                    result.errors += 1;
                }
            }
        }
        Ok(())
    }

    fn should_skip_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check against exclude patterns
        for pattern_str in &self.config.exclude_patterns {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                if pattern.matches(&path_str) {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Default)]
pub struct WalkResult {
    pub files_processed: usize,
    pub files_skipped: usize,
    pub total_changes: usize,
    pub errors: usize,
}

impl WalkResult {
    pub fn print_summary(&self, dry_run: bool) {
        if dry_run {
            println!("\nDry run summary:");
            println!("  Files that would be processed: {}", self.files_processed);
            println!("  Invisible characters that would be removed: {}", self.total_changes);
        } else {
            println!("\nProcessing summary:");
            println!("  Files processed: {}", self.files_processed);
            println!("  Invisible characters removed: {}", self.total_changes);
        }

        if self.files_skipped > 0 {
            println!("  Files skipped: {}", self.files_skipped);
        }

        if self.errors > 0 {
            println!("  Errors encountered: {}", self.errors);
        }
    }
}