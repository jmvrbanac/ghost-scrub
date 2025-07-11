# 👻 Ghost Scrub

A powerful Rust CLI tool that strips invisible characters from text and code files that AI LLMs sometimes add during code generation.

## 🎯 What It Does

Ghost Scrub removes problematic invisible characters that can break your code or cause mysterious issues:

- **Zero-width spaces** (U+200B, U+200C, U+200D, U+FEFF)
- **Non-breaking spaces** (U+00A0)
- **Control characters** (ASCII 0x00-0x1F, 0x7F)
- **Unicode whitespace** characters
- **Whitespace-only lines** (converts to empty lines)
- **Custom Unicode characters** (configurable)

## 🚀 Installation

### Download Pre-built Binaries (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/jmvrbanac/ghost-scrub/releases):

- **Linux (x86_64)**: `ghost-scrub-v0.1.1-x86_64-unknown-linux-gnu.tar.gz`
- **macOS (Intel)**: `ghost-scrub-v0.1.1-x86_64-apple-darwin.tar.gz`
- **macOS (Apple Silicon)**: `ghost-scrub-v0.1.1-aarch64-apple-darwin.tar.gz`
- **Windows**: `ghost-scrub-v0.1.1-x86_64-pc-windows-msvc.zip`

```bash
# Example for Linux:
wget https://github.com/jmvrbanac/ghost-scrub/releases/latest/download/ghost-scrub-v0.1.1-x86_64-unknown-linux-gnu.tar.gz
tar -xzf ghost-scrub-v0.1.1-x86_64-unknown-linux-gnu.tar.gz
sudo mv ghost-scrub /usr/local/bin/

# Example for macOS (Intel):
curl -L https://github.com/jmvrbanac/ghost-scrub/releases/latest/download/ghost-scrub-v0.1.1-x86_64-apple-darwin.tar.gz | tar -xz
sudo mv ghost-scrub /usr/local/bin/

# Example for macOS (Apple Silicon):
curl -L https://github.com/jmvrbanac/ghost-scrub/releases/latest/download/ghost-scrub-v0.1.1-aarch64-apple-darwin.tar.gz | tar -xz
sudo mv ghost-scrub /usr/local/bin/
```

### Build from Source

```bash
# Clone and build from source
git clone https://github.com/jmvrbanac/ghost-scrub.git
cd ghost-scrub
cargo build --release

# The binary will be in target/release/ghost-scrub
```

### Install via Cargo

```bash
# Install directly from GitHub
cargo install --git https://github.com/jmvrbanac/ghost-scrub.git

# Or install from local source
cargo install --path .
```

## 📖 Usage

### Basic Usage
```bash
# Process current directory recursively
ghost-scrub

# Process specific files or directories
ghost-scrub src/ main.rs

# Dry run to see what would be changed
ghost-scrub --dry-run

# Verbose output with detailed diffs
ghost-scrub --verbose

# Watch mode for real-time processing
ghost-scrub --watch src/
```

### Configuration
```bash
# Create default configuration file
ghost-scrub init

# Use custom config file
ghost-scrub --config my-config.toml
```

### Command Options
```
Usage: ghost-scrub [OPTIONS] [PATH]... [COMMAND]

Commands:
  init  Create a default .ghostscrub configuration file

Arguments:
  [PATH]...  Files or directories to process (defaults to current directory)

Options:
  -n, --dry-run        Show what would be changed without modifying files
  -w, --watch          Watch directories for changes and process files automatically
  -c, --config <FILE>  Path to configuration file (defaults to .ghostscrub)
  -v, --verbose        Show detailed output including diffs of changes
  -h, --help           Print help
  -V, --version        Print version
```

## ⚙️ Configuration

Create a `.ghostscrub` configuration file to customize behavior:

```toml
# File extensions to include
include_extensions = [
    "rs", "py", "js", "ts", "jsx", "tsx", "go", "java", "c", "cpp", "h", "hpp",
    "cs", "php", "rb", "swift", "kt", "scala", "clj", "hs", "ml",
    "txt", "md", "json", "xml", "yaml", "yml", "toml", "ini", "cfg", "conf"
]

# File extensions to exclude
exclude_extensions = []

# Glob patterns to include/exclude
include_patterns = ["**/*"]
exclude_patterns = ["**/target/**", "**/node_modules/**"]

# Configure which invisible characters to target
[target_characters]
zero_width_spaces = true      # U+200B, U+200C, U+200D, U+FEFF
non_breaking_spaces = true    # U+00A0
control_characters = true     # ASCII control chars (0x00-0x1F, 0x7F)
unicode_whitespace = true     # Other Unicode whitespace characters
custom_chars = []             # Additional specific Unicode characters to remove

# Verbosity level: "silent", "normal", "verbose"
verbosity = "normal"
```

## 📊 Verbose Output

When using `--verbose`, ghost-scrub shows detailed diffs of changes:

```
--- Original
+++ Cleaned
-42: console.log("HelloWorld");  ⦃ZWS⦄
+42: console.log("HelloWorld");
-105: ⦃WHITESPACE-ONLY: SP+TAB+SP⦄
+105: ⦃EMPTY⦄
```

### Invisible Character Visualization
- `⦃ZWS⦄` - Zero Width Space
- `⦃ZWNJ⦄` - Zero Width Non-Joiner
- `⦃ZWJ⦄` - Zero Width Joiner
- `⦃BOM⦄` - Byte Order Mark
- `⦃NBSP⦄` - Non-Breaking Space
- `⦃TAB⦄` - Tab character
- `⦃WHITESPACE-ONLY: SP+TAB⦄` - Lines with only whitespace
- `⦃EMPTY⦄` - Truly empty lines
- `⦃U+XXXX⦄` - Other Unicode characters

## 🔍 Watch Mode

Ghost Scrub can monitor directories and automatically clean files as they're modified:

```bash
# Watch current directory
ghost-scrub --watch

# Watch specific directories
ghost-scrub --watch src/ tests/
```

Watch mode automatically skips temporary files (`.tmp`, `.swp`, `.#*`, etc.) and hidden files.

## 🎯 Use Cases

- **AI-Generated Code**: Clean up invisible characters that LLMs sometimes insert
- **Copy-Paste Issues**: Remove problematic characters from copied text
- **Cross-Platform Development**: Normalize whitespace across different systems
- **Code Review**: Ensure clean, consistent formatting
- **CI/CD Integration**: Automated cleaning in build pipelines

## 🔧 Integration Examples

### Git Pre-commit Hook
```bash
#!/bin/sh
ghost-scrub --dry-run
if [ $? -ne 0 ]; then
  echo "Ghost Scrub found issues. Run 'ghost-scrub' to fix them."
  exit 1
fi
```

### VS Code Task
```json
{
  "label": "Ghost Scrub",
  "type": "shell",
  "command": "ghost-scrub",
  "args": ["--verbose", "${workspaceFolder}"],
  "group": "build"
}
```

### NeoVim Integration
```lua
-- Add to your init.lua or a plugin file
vim.api.nvim_create_user_command('GhostScrub', function(opts)
  local cmd = {'ghost-scrub'}
  if opts.bang then
    table.insert(cmd, '--dry-run')
  end
  if opts.args ~= '' then
    table.insert(cmd, opts.args)
  else
    table.insert(cmd, vim.fn.expand('%'))
  end

  vim.fn.jobstart(cmd, {
    on_stdout = function(_, data)
      for _, line in ipairs(data) do
        if line ~= '' then
          print(line)
        end
      end
    end,
  })
end, {
  bang = true,  -- :GhostScrub! for dry-run
  nargs = '?',  -- Optional file/directory argument
  desc = 'Run Ghost Scrub on current file or specified path'
})

-- Optional keybinding
vim.keymap.set('n', '<leader>gs', ':GhostScrub<CR>', { desc = 'Ghost Scrub current file' })

-- Optional: Run on save
-- Set this to true to enable auto-run on save
local ghost_scrub_on_save = false

if ghost_scrub_on_save then
  vim.api.nvim_create_autocmd('BufWritePost', {
    pattern = '*',
    callback = function()
      vim.cmd('GhostScrub')
    end,
    desc = 'Run Ghost Scrub after saving'
  })
end

-- Or use a global variable to toggle it dynamically:
-- vim.g.ghost_scrub_auto = true  -- Enable
-- vim.g.ghost_scrub_auto = false -- Disable
-- Then toggle with: :let g:ghost_scrub_auto = v:true
```

## 🛡️ Safety Features

- **Dry-run mode**: Preview changes before applying them
- **File extension filtering**: Only process specified file types
- **Directory exclusions**: Skip build artifacts and dependencies
- **Backup-friendly**: Works with version control for easy rollback
- **Non-destructive**: Preserves file permissions and timestamps

## 🏗️ Building from Source

```bash
# Requirements: Rust 1.70+
git clone https://github.com/jmvrbanac/ghost-scrub.git
cd ghost-scrub
cargo build --release

# Run tests
cargo test

# Install globally
cargo install --path .
```

## 📋 File Support

Ghost Scrub processes text-based files by default, including:

- **Programming languages**: Rust, Python, JavaScript, TypeScript, Go, Java, C/C++, C#, PHP, Ruby, Swift, Kotlin, Scala, Clojure, Haskell, ML
- **Configuration files**: JSON, YAML, TOML, INI, XML
- **Documentation**: Markdown, plain text
- **Custom extensions**: Configurable via `.ghostscrub`

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `cargo test` and `cargo clippy` pass
5. Submit a pull request

## 📄 License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## 🔗 Related Tools

- [prettier](https://prettier.io/) - Code formatting
- [editorconfig](https://editorconfig.org/) - Consistent coding styles
- [dos2unix](https://dos2unix.sourceforge.io/) - Line ending conversion

---

**Ghost Scrub** - Because invisible problems need visible solutions! 👻✨