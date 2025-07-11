use crate::config::{GhostScrubConfig, VerbosityLevel};
use std::fs;
use std::path::Path;

pub struct FileProcessor {
    config: GhostScrubConfig,
}

impl FileProcessor {
    pub fn new(config: GhostScrubConfig) -> Self {
        Self { config }
    }

    pub fn process_file(
        &self,
        file_path: &Path,
        dry_run: bool,
        verbose: bool,
    ) -> Result<ProcessResult, Box<dyn std::error::Error>> {
        if !self.config.should_process_file(file_path) {
            return Ok(ProcessResult::Skipped);
        }

        let content = fs::read_to_string(file_path)?;
        let cleaned_content = self.clean_content(&content);

        if content == cleaned_content {
            if matches!(self.config.verbosity, VerbosityLevel::Verbose) {
                println!("No changes needed: {}", file_path.display());
            }
            return Ok(ProcessResult::NoChanges);
        }

        let changes_count = self.count_changes(&content, &cleaned_content);

        if verbose {
            self.print_diff(file_path, &content, &cleaned_content, dry_run);
        }

        if dry_run {
            if !verbose {
                println!(
                    "Would clean {} invisible characters from: {}",
                    changes_count,
                    file_path.display()
                );
            }
            Ok(ProcessResult::DryRun(changes_count))
        } else {
            fs::write(file_path, cleaned_content)?;
            if !matches!(self.config.verbosity, VerbosityLevel::Silent) && !verbose {
                println!(
                    "Cleaned {} invisible characters from: {}",
                    changes_count,
                    file_path.display()
                );
            }
            Ok(ProcessResult::Cleaned(changes_count))
        }
    }

    fn clean_content(&self, content: &str) -> String {
        let mut result = content.to_string();

        if self.config.target_characters.zero_width_spaces {
            result = self.remove_zero_width_spaces(&result);
        }

        if self.config.target_characters.non_breaking_spaces {
            result = self.remove_non_breaking_spaces(&result);
        }

        if self.config.target_characters.control_characters {
            result = self.remove_control_characters(&result);
        }

        if self.config.target_characters.unicode_whitespace {
            result = self.remove_unicode_whitespace(&result);
        }

        if self.config.target_characters.trailing_whitespace {
            result = self.remove_trailing_whitespace(&result);
        }

        for custom_char in &self.config.target_characters.custom_chars {
            if let Ok(unicode_char) = u32::from_str_radix(custom_char.trim_start_matches("U+"), 16)
            {
                if let Some(ch) = char::from_u32(unicode_char) {
                    result = result.replace(ch, "");
                }
            }
        }

        // Remove lines that contain only whitespace (spaces, tabs)
        result = self.remove_whitespace_only_lines(&result);

        result
    }

    fn remove_zero_width_spaces(&self, content: &str) -> String {
        content.replace(['\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}'], "") // Zero Width No-Break Space (BOM)
    }

    fn remove_non_breaking_spaces(&self, content: &str) -> String {
        content.replace('\u{00A0}', " ") // Non-Breaking Space -> regular space
    }

    fn remove_control_characters(&self, content: &str) -> String {
        content
            .chars()
            .filter(|&ch| {
                // Keep newlines, carriage returns, and tabs
                if ch == '\n' || ch == '\r' || ch == '\t' {
                    return true;
                }
                // Remove other ASCII control characters
                !(ch as u32 <= 0x1F || ch as u32 == 0x7F)
            })
            .collect()
    }

    fn remove_unicode_whitespace(&self, content: &str) -> String {
        content
            .chars()
            .filter(|&ch| {
                // Keep normal spaces, newlines, carriage returns, and tabs
                if ch == ' ' || ch == '\n' || ch == '\r' || ch == '\t' {
                    return true;
                }
                // Remove other Unicode whitespace characters
                !ch.is_whitespace()
            })
            .collect()
    }

    fn remove_trailing_whitespace(&self, content: &str) -> String {
        content
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn remove_whitespace_only_lines(&self, content: &str) -> String {
        content
            .lines()
            .map(|line| {
                if line.trim().is_empty() {
                    // Keep the newline but remove all whitespace
                    ""
                } else {
                    line
                }
            })
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn print_diff(&self, file_path: &Path, original: &str, cleaned: &str, dry_run: bool) {
        let action = if dry_run { "Would clean" } else { "Cleaned" };
        let changes_count = self.count_changes(original, cleaned);

        println!(
            "{} {} invisible characters from: {}",
            action,
            changes_count,
            file_path.display()
        );

        if changes_count == 0 {
            return;
        }

        println!("--- Original");
        println!("+++ Cleaned");

        let original_lines: Vec<&str> = original.lines().collect();
        let cleaned_lines: Vec<&str> = cleaned.lines().collect();

        let max_lines = original_lines.len().max(cleaned_lines.len());

        for i in 0..max_lines {
            let orig_line = original_lines.get(i).unwrap_or(&"");
            let clean_line = cleaned_lines.get(i).unwrap_or(&"");

            if orig_line != clean_line {
                println!("-{}: {}", i + 1, self.visualize_invisible_chars(orig_line));
                println!("+{}: {}", i + 1, self.visualize_invisible_chars(clean_line));
            }
        }
        println!();
    }

    fn visualize_invisible_chars(&self, text: &str) -> String {
        if text.trim().is_empty() && !text.is_empty() {
            // Line contains only whitespace - show each character
            format!(
                "⦃WHITESPACE-ONLY: {}⦄",
                text.chars()
                    .map(|ch| match ch {
                        ' ' => "SP".to_string(),
                        '\t' => "TAB".to_string(),
                        '\u{00A0}' => "NBSP".to_string(),
                        ch if ch.is_whitespace() => format!("WS:U+{:04X}", ch as u32),
                        ch => format!("U+{:04X}", ch as u32),
                    })
                    .collect::<Vec<_>>()
                    .join("+")
            )
        } else if text.is_empty() {
            "⦃EMPTY⦄".to_string()
        } else {
            // Check for trailing whitespace
            let has_trailing_whitespace = text.len() != text.trim_end().len();
            let main_content = text
                .chars()
                .map(|ch| match ch {
                    '\u{200B}' => "⦃ZWS⦄".to_string(),
                    '\u{200C}' => "⦃ZWNJ⦄".to_string(),
                    '\u{200D}' => "⦃ZWJ⦄".to_string(),
                    '\u{FEFF}' => "⦃BOM⦄".to_string(),
                    '\u{00A0}' => "⦃NBSP⦄".to_string(),
                    '\t' => "⦃TAB⦄".to_string(),
                    ' ' => " ".to_string(), // Keep regular spaces visible
                    ch if ch.is_control() && ch != '\n' && ch != '\r' => {
                        format!("⦃U+{:04X}⦄", ch as u32)
                    }
                    ch if ch.is_whitespace()
                        && ch != ' '
                        && ch != '\n'
                        && ch != '\r'
                        && ch != '\t' =>
                    {
                        format!("⦃WS:U+{:04X}⦄", ch as u32)
                    }
                    ch => ch.to_string(),
                })
                .collect::<String>();

            if has_trailing_whitespace {
                let trailing_chars: String = text
                    .chars()
                    .skip(text.trim_end().len())
                    .map(|ch| match ch {
                        ' ' => "SP".to_string(),
                        '\t' => "TAB".to_string(),
                        '\u{00A0}' => "NBSP".to_string(),
                        ch if ch.is_whitespace() => format!("WS:U+{:04X}", ch as u32),
                        ch => format!("U+{:04X}", ch as u32),
                    })
                    .collect::<Vec<_>>()
                    .join("+");
                format!("{}⦃TRAILING: {}⦄", text.trim_end(), trailing_chars)
            } else {
                main_content
            }
        }
    }

    fn count_changes(&self, original: &str, cleaned: &str) -> usize {
        original.len() - cleaned.len()
    }
}

#[derive(Debug)]
pub enum ProcessResult {
    Cleaned(usize),
    DryRun(usize),
    NoChanges,
    Skipped,
}