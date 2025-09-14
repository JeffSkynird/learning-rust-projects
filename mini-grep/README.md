# mini-grep

Tiny text searcher (a mini grep) written in Rust.  
Supports **regex**, **recursive search**, **line numbers**, **whole-word matches**, **ANSI color**, and respects **.gitignore** (when using the version with the `ignore` crate).

---

## Installation

```bash
# In an empty folder:
cargo init --bin mini-grep
# Replace Cargo.toml and src/main.rs with your code
cargo build
```

> Requires Rust toolchain (`cargo`, `rustc`). Install from https://rustup.rs

---

## Usage

```
mini-grep [OPTIONS] <PATTERN> [PATH]...

Options:
  -r, --recursive          search directories recursively
  -n, --line-number        show line number
  -i, --ignore-case        case-insensitive
  -w, --word               whole-word match (\b(?:PATTERN)\b)
      --no-color           disable ANSI colors
  -m, --max-count <N>      stop after N total matches
      --hidden             include hidden files (e.g. .gitignore, .env)
      --no-ignore          do NOT respect .gitignore/.ignore
      --glob <GLOB>        include-only glob(s), can repeat (e.g. --glob "**/*.rs")
      --binary             include binary files (skipped by default)
  -V, --version
  -h, --help

Arguments:
  <PATTERN>                regex pattern to search
  [PATH]...                files/dirs to search (default: .)
```

**Exit codes:** `0` = at least one match, `1` = no matches, `2` = error (I/O, bad regex, …)

---

## Quick start

```bash
# Search the current project (recursively), case-insensitive, line number
mini-grep -r -i -n "todo|fixme" .

# Only Rust sources, show line numbers
mini-grep -r -n --glob "**/*.rs" "panic|error" .

# Exact whole word "main" (won't match "domain"/"maintain")
mini-grep -r -w main .

# Stop after the first 5 matches
mini-grep -r -m 5 "fn\s+\w+\(" .

# Search only in src/ (will NOT touch target/ if you pass src explicitly)
mini-grep -r -i "panic|error" src
```

> **Note:** If you see tons of hits under `target/`, either:
> - search only `src` (`mini-grep -r pattern src`), or  
> - rely on `.gitignore` (default if you used the version with the `ignore` crate), or  
> - explicitly exclude using a glob negation: `--glob "!**/target/**"`.

---

## Examples (copy & try)

### Basics
```bash
# Find literal text "unsafe" (regex is on by default; literals also work)
mini-grep -r unsafe .

# Multiple directories
mini-grep -r "TODO" src tests benches
```

### Case sensitivity & whole words
```bash
# Case-insensitive (matches "Error", "error", "ERROR", ...)
mini-grep -r -i "error" .

# Whole word "Result"
mini-grep -r -w "Result" src
```

### Line numbers & colors
```bash
# Show line & column, with ANSI highlighting
mini-grep -r -n "unwrap\(" .

# Disable color (useful when piping to files)
mini-grep -r --no-color "TODO" .
```

### Glob filtering
```bash
# Only Rust files
mini-grep -r --glob "**/*.rs" "panic|error" .

# Only Markdown & TOML
mini-grep -r --glob "**/*.md" --glob "**/*.toml" "(roadmap|changelog)" .

# Exclude a directory (negated glob)
mini-grep -r --glob "!**/target/**" --glob "**/*" "error" .
```

### Respecting `.gitignore` / hidden files / binary files
```bash
# Default: respects .gitignore and skips binary-like files
mini-grep -r "panic" .

# Include hidden files
mini-grep -r --hidden "secret" .

# Ignore .gitignore rules (search absolutely everything)
mini-grep -r --no-ignore "panic" .

# Include binaries (off by default). Use sparingly.
mini-grep -r --binary "ELF" .
```

### Practical Rust patterns
```bash
# Function definitions (fn followed by name and an opening paren)
mini-grep -r -n 'fn\s+\w+\s*\(' src

# unwrap/expect usage
mini-grep -r -n 'unwrap\(|expect\(' src

# Trait impls: impl <Trait> for <Type>
mini-grep -r -n 'impl\s+\w+\s+for\s+\w+' src

# log/error macros
mini-grep -r -n '(eprintln!|log::error!|tracing::error!)' src

# Find TODOs / FIXMEs
mini-grep -r -n -i 'TODO|FIXME|HACK' .
```

### Regex features & anchors
```bash
# Lines starting with use
mini-grep -r '^\s*use\s' src

# Lines ending with a semicolon
mini-grep -r ';\s*$' src

# Alternate between "panic" OR "error"
mini-grep -r '(panic|error)' src
```

> The tool reads **line by line**, so `.` does **not** span across newlines. `^` and `$` anchor to each **line**.

### Limiting output
```bash
# First 10 matches (then stop)
mini-grep -r -m 10 "." src

# Count matches (shell trick)
mini-grep -r 'TODO' . | wc -l          # Linux/macOS
mini-grep -r 'TODO' . | Measure-Object -Line   # PowerShell
```

### Multiple paths
```bash
# Search in src/ and tests/ only
mini-grep -r "Result" src tests
```

### Windows quoting tips
```powershell
# Use double quotes in PowerShell
mini-grep -r -i "panic|error" .
mini-grep -r -n "fn\s+\w+\(" src
```

---

## Output format

```
path/to/file:LINE:COLUMN: line content with matches highlighted
```

- **COLUMN** counts **characters** (not bytes), so Unicode is handled correctly.
- When `-n` is omitted: `path/to/file:COLUMN: line content…`

---

## Performance tips

- Prefer **scope first** (`src` instead of `.`) to avoid `target/`, `node_modules/`, etc.
- Use `--glob` to narrow file types (`"**/*.rs"`, `"**/*.md"`).
- Combine `-m` to stop early when you just need a few hits.

---

## Troubleshooting

- “It searches `target/` and prints a flood of stuff”  
  → Run from the project root with `.gitignore` in place (the tool respects it), or search `src` only:  
  `mini-grep -r pattern src`  
  You can also exclude explicitly: `--glob "!**/target/**" --glob "**/*"`.

- “Regex escapes look weird”  
  Remember shells treat backslashes. In bash/zsh use **single quotes**: `'fn\s+\w+\('`.  
  In PowerShell use **double quotes**: `"fn\s+\w+\("`.

---

## Exit codes recap

- `0` → at least one match found  
- `1` → no matches  
- `2` → error (I/O, regex compile error, traversal error, …)
