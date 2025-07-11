use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostScrubConfig {
    #[serde(default = "default_include_extensions")]
    pub include_extensions: Vec<String>,

    #[serde(default)]
    pub exclude_extensions: Vec<String>,

    #[serde(default = "default_include_patterns")]
    pub include_patterns: Vec<String>,

    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    #[serde(default = "default_target_chars")]
    pub target_characters: TargetCharacters,

    #[serde(default = "default_verbosity")]
    pub verbosity: VerbosityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetCharacters {
    #[serde(default = "default_true")]
    pub zero_width_spaces: bool,

    #[serde(default = "default_true")]
    pub non_breaking_spaces: bool,

    #[serde(default = "default_true")]
    pub control_characters: bool,

    #[serde(default = "default_true")]
    pub unicode_whitespace: bool,

    #[serde(default = "default_true")]
    pub trailing_whitespace: bool,

    #[serde(default)]
    pub custom_chars: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerbosityLevel {
    Silent,
    Normal,
    Verbose,
}

impl Default for GhostScrubConfig {
    fn default() -> Self {
        Self {
            include_extensions: default_include_extensions(),
            exclude_extensions: Vec::new(),
            include_patterns: default_include_patterns(),
            exclude_patterns: default_exclude_patterns(),
            target_characters: default_target_chars(),
            verbosity: default_verbosity(),
        }
    }
}

impl Default for TargetCharacters {
    fn default() -> Self {
        default_target_chars()
    }
}

impl Default for VerbosityLevel {
    fn default() -> Self {
        default_verbosity()
    }
}

fn default_include_extensions() -> Vec<String> {
    vec![
        "rs".to_string(),
        "py".to_string(),
        "js".to_string(),
        "ts".to_string(),
        "jsx".to_string(),
        "tsx".to_string(),
        "go".to_string(),
        "java".to_string(),
        "c".to_string(),
        "cpp".to_string(),
        "h".to_string(),
        "hpp".to_string(),
        "cs".to_string(),
        "php".to_string(),
        "rb".to_string(),
        "swift".to_string(),
        "kt".to_string(),
        "scala".to_string(),
        "clj".to_string(),
        "hs".to_string(),
        "ml".to_string(),
        "txt".to_string(),
        "md".to_string(),
        "json".to_string(),
        "xml".to_string(),
        "yaml".to_string(),
        "yml".to_string(),
        "toml".to_string(),
        "ini".to_string(),
        "cfg".to_string(),
        "conf".to_string(),
    ]
}

fn default_include_patterns() -> Vec<String> {
    vec!["**/*".to_string()]
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        // Version control
        "**/.git/**".to_string(),
        "**/.svn/**".to_string(),
        "**/.hg/**".to_string(),
        "**/.bzr/**".to_string(),
        // Build artifacts and dependencies
        "**/target/**".to_string(),
        "**/node_modules/**".to_string(),
        "**/build/**".to_string(),
        "**/dist/**".to_string(),
        "**/out/**".to_string(),
        "**/bin/**".to_string(),
        "**/obj/**".to_string(),
        // Python
        "**/__pycache__/**".to_string(),
        "**/.pytest_cache/**".to_string(),
        "**/venv/**".to_string(),
        "**/.venv/**".to_string(),
        "**/*.egg-info/**".to_string(),
        // IDEs and editors
        "**/.idea/**".to_string(),
        "**/.vscode/**".to_string(),
        "**/.vs/**".to_string(),
        "**/*.swp".to_string(),
        "**/*.swo".to_string(),
        "**/*~".to_string(),
        "**/.#*".to_string(),
        // OS specific
        "**/.DS_Store".to_string(),
        "**/Thumbs.db".to_string(),
        "**/desktop.ini".to_string(),
        // Temporary files
        "**/*.tmp".to_string(),
        "**/*.temp".to_string(),
        "**/*.bak".to_string(),
        "**/*.orig".to_string(),
        // Logs
        "**/*.log".to_string(),
        "**/logs/**".to_string(),
    ]
}

fn default_target_chars() -> TargetCharacters {
    TargetCharacters {
        zero_width_spaces: true,
        non_breaking_spaces: true,
        control_characters: true,
        unicode_whitespace: true,
        trailing_whitespace: true,
        custom_chars: Vec::new(),
    }
}

fn default_verbosity() -> VerbosityLevel {
    VerbosityLevel::Normal
}

fn default_true() -> bool {
    true
}

impl GhostScrubConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: GhostScrubConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_default() -> Self {
        Self::load_from_file(".ghostscrub").unwrap_or_default()
    }

    pub fn should_process_file(&self, file_path: &Path) -> bool {
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            if !self.exclude_extensions.is_empty()
                && self.exclude_extensions.contains(&extension.to_string())
            {
                return false;
            }

            if !self.include_extensions.is_empty()
                && !self.include_extensions.contains(&extension.to_string())
            {
                return false;
            }
        }

        true
    }
}