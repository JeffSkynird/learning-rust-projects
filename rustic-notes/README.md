# rustic-notes — a tiny notes CLI in Rust (JSON/TOML/YAML)

**rustic-notes** is a learning-friendly command‑line notes manager written in Rust. It stores your notes in a single file using your preferred format — **JSON**, **TOML**, or **YAML** — powered by **serde** for serialization/deserialization.


## Features
- Add, list, search, remove, and edit notes from the terminal.
- Choose the storage format: `json`, `toml`, or `yaml`.
- Open your note in `$VISUAL` / `$EDITOR` (`--open-editor`) for rich editing.
- Tags with normalization and case‑insensitive deduplication.
- Timestamps via `chrono`.

---

## Quick start
```bash
# Build
cargo build --release

# Add a note (defaults to JSON -> notes.json)
./target/release/rustic-notes add "Buy bread" -b "This afternoon" -t personal,errands

# List
./target/release/rustic-notes list -l
```

If you prefer TOML/YAML or a custom path:
```bash
./target/release/rustic-notes --format toml --store data/notes.toml add "App idea" -b "Hack a prototype" -t ideas,dev
./target/release/rustic-notes --format toml --store data/notes.toml list
./target/release/rustic-notes --format yaml list -l
```

---

## Usage
```
rustic-notes [--format <json|toml|yaml>] [--store <PATH>] <COMMAND>
```

### Global options
- `-f, --format <json|toml|yaml>`  Storage format (default: `json`).
- `-s, --store <PATH>`             File path (default: `notes.json|toml|yaml` depending on `--format`).

### Commands
- `add <title> [-b, --body <text>] [-t, --tags tag1,tag2]`
- `list [-l, --long]`
- `search <query> [-t, --tags tag1,tag2]`
- `remove <id>`
- `edit <id> [--title <t>] [--body <b>] [--tags tag1,tag2] [--add-tags a,b] [--rm-tags r,b] [--open-editor] [--editor-format <yaml|json|toml>]`

### Examples
```bash
# Add
rustic-notes add "Read book" -b "Ch. 3 & 4" -t reading

# List (headers only)
rustic-notes list

# List with bodies
rustic-notes list -l

# Search by text
rustic-notes search bread

# Search by text and require tags
rustic-notes search idea -t dev,ideas

# Remove by id
rustic-notes remove 3

# Edit by flags
rustic-notes edit 2 --title "New title" --body "New body"
rustic-notes edit 2 --tags work,ideas
rustic-notes edit 2 --add-tags backend --rm-tags ideas

# Edit in your editor (YAML temp file by default)
VISUAL="code -w" rustic-notes edit 2 --open-editor
# Choose editor file format
EDITOR=nvim rustic-notes edit 2 --open-editor --editor-format json
```

---

## Editing with `$VISUAL` / `$EDITOR`
- The tool launches `$VISUAL` if set, otherwise `$EDITOR`; if neither is set, it falls back to `vi` (Unix) or `notepad` (Windows).
- For VS Code, use `code -w` so the CLI **waits** until you close the editor.
- The temporary file contains a minimal editable object with `title`, `body`, and `tags`. Remove a field to keep the current value, or edit it to apply changes.

---

## Search & tags semantics
- **Search** is case‑insensitive and matches substrings in `title`, `body`, or `tags`.
- `-t, --tags` in `search` requires **all** the given tags to be present (logical AND).
- **Tags** are trimmed, empty ones are removed, and deduplicated case‑insensitively (so `Dev` and `dev` are treated as the same tag).

---

## Storage & schema
By default the store file is `notes.json`, `notes.toml`, or `notes.yaml` depending on `--format`. You can change the path with `--store`.

Schema (conceptual):
```text
Storage {
  notes: Vec<Note>
}

Note {
  id: u64,
  title: String,
  body: String,
  tags: Vec<String>,
  created_at: RFC3339 timestamp (UTC)
}
```

### Example files
**JSON**
```json
{
  "notes": [
    {
      "id": 1,
      "title": "Buy bread",
      "body": "This afternoon",
      "tags": ["personal", "errands"],
      "created_at": "2025-09-14T12:34:56Z"
    }
  ]
}
```

**TOML**
```toml
[[notes]]
id = 1
title = "Buy bread"
body = "This afternoon"
tags = ["personal", "errands"]
created_at = 2025-09-14T12:34:56Z
```

**YAML**
```yaml
notes:
  - id: 1
    title: Buy bread
    body: This afternoon
    tags: [personal, errands]
    created_at: 2025-09-14T12:34:56Z
```

> You can edit the store file by hand if you keep it valid JSON/TOML/YAML.

## Development
**Dependencies** (from `Cargo.toml`):
- `clap` (CLI parsing with derive)
- `serde`, `serde_json`, `serde_yaml`, `toml` (serialization)
- `chrono` (timestamps; RFC3339 via `serde` feature)
- `anyhow` (ergonomic error handling)
- `tempfile` (editor temp files)
- `shell-words` (parse `$EDITOR` like `"code -w"`)

Build & run:
```bash
cargo run -- add "First note"
cargo build --release
```

Run the binary directly after a release build:
```bash
./target/release/rustic-notes list
```

---

## Troubleshooting
- **Editor doesn’t return**: with VS Code set `VISUAL="code -w"` (the `-w` makes it wait). For Vim/Neovim/Nano this is not needed.
- **Unknown editor**: set `VISUAL` or `EDITOR` to a valid command, e.g. `export VISUAL=nvim` (Unix) or set environment variables in Windows.
- **YAML/JSON/TOML parse error** after editing: ensure the file remains valid; try `--editor-format json` if your editor has better JSON tooling.