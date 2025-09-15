use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::{env, fs, io::Write, path::{Path, PathBuf}, process::Command as ProcCommand};
use std::collections::HashSet;
use tempfile::Builder as TempBuilder;

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
enum Format {
    Json,
    Toml,
    Yaml,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
enum EditorFmt {
    Json,
    Toml,
    Yaml,
}

#[derive(Debug, Parser)]
#[command(name = "rustic-notes", version, about = "Note manager (JSON/TOML/YAML) with serde")]
struct Cli {
    /// Storage format: json | toml | yaml
    #[arg(short = 'f', long = "format", value_enum, default_value_t = Format::Json)]
    format: Format,

    /// File path for the notes (default: notes.{json|toml|yaml})
    #[arg(short = 's', long = "store")]
    store: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Adds a new note
    Add {
        title: String,
        /// Body of the note (use -b "long text")
        #[arg(short = 'b', long = "body", default_value = "")]
        body: String,
        /// Tags separated by commas, e.g.: -t work,ideas
        #[arg(short = 't', long = "tags", value_delimiter = ',')]
        tags: Vec<String>,
    },

    /// List all notes
    List {
        /// Shows also the body
        #[arg(short = 'l', long = "long")]
        long: bool,
    },

    /// Search notes by text and/or tags
    Search {
        /// Text to search (in title, body or tags)
        query: String,
        /// Require the note to contain ALL these tags (comma-separated)
        #[arg(short = 't', long = "tags", value_delimiter = ',')]
        tags: Vec<String>,
    },

    /// Delete a note by id
    Remove { id: u64 },

    /// Edit note fields by id
    Edit {
        /// ID
        id: u64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        #[arg(long = "add-tags", value_delimiter = ',')]
        add_tags: Vec<String>,
        #[arg(long = "rm-tags", value_delimiter = ',')]
        rm_tags: Vec<String>,
        /// Opens a temporary file in the editor defined in $VISUAL or $EDITOR to edit YAML/JSON/TOML
        #[arg(long = "open-editor", default_value_t = false)]
        open_editor: bool,
        /// Format of the temporary file opened in the editor
        #[arg(long = "editor-format", value_enum, default_value_t = EditorFmt::Yaml)]
        editor_format: EditorFmt,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Storage {
    notes: Vec<Note>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Note {
    id: u64,
    title: String,
    body: String,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct EditableNote {
    title: Option<String>,
    body: Option<String>,
    tags: Option<Vec<String>>, // if omitted, they remain
}

impl Storage {
    fn next_id(&self) -> u64 {
        self.notes.iter().map(|n| n.id).max().unwrap_or(0) + 1
    }
}

fn default_store_for(format: Format) -> &'static str {
    match format {
        Format::Json => "notes.json",
        Format::Toml => "notes.toml",
        Format::Yaml => "notes.yaml",
    }
}

fn load(path: &Path, format: Format) -> anyhow::Result<Storage> {
    if !path.exists() {
        return Ok(Storage::default());
    }
    let raw = fs::read_to_string(path)?;
    if raw.trim().is_empty() {
        return Ok(Storage::default());
    }
    let storage = match format {
        Format::Json => serde_json::from_str(&raw)?,
        Format::Toml => toml::from_str(&raw)?,
        Format::Yaml => serde_yaml::from_str(&raw)?,
    };
    Ok(storage)
}

fn save(path: &Path, format: Format, storage: &Storage) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let raw = match format {
        Format::Json => serde_json::to_string_pretty(storage)?,
        Format::Toml => toml::to_string_pretty(storage)?,
        Format::Yaml => serde_yaml::to_string(storage)?,
    };
    fs::write(path, raw)?;
    Ok(())
}

fn normalize_tags(mut tags: Vec<String>) -> Vec<String> {
    // Trim + dedup (case-insensitive), without empty
    tags.iter_mut().for_each(|t| *t = t.trim().to_string());
    tags.retain(|t| !t.is_empty());
    let mut seen = HashSet::new();
    tags.into_iter()
        .filter(|t| seen.insert(t.to_lowercase()))
        .collect()
}

fn add_tags(existing: &mut Vec<String>, additions: Vec<String>) {
    let to_add = normalize_tags(additions);
    let mut seen: HashSet<String> = existing.iter().map(|t| t.to_lowercase()).collect();
    for t in to_add {
        if seen.insert(t.to_lowercase()) {
            existing.push(t);
        }
    }
}

fn remove_tags(existing: &mut Vec<String>, removals: Vec<String>) {
    let remset: HashSet<String> = removals
        .into_iter()
        .map(|s| s.to_lowercase().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    existing.retain(|t| !remset.contains(&t.to_lowercase()));
}

fn editable_from_note(n: &Note) -> EditableNote {
    EditableNote {
        title: Some(n.title.clone()),
        body: Some(n.body.clone()),
        tags: Some(n.tags.clone()),
    }
}

fn to_text(fmt: EditorFmt, e: &EditableNote) -> anyhow::Result<String> {
    Ok(match fmt {
        EditorFmt::Json => serde_json::to_string_pretty(e)?,
        EditorFmt::Toml => toml::to_string_pretty(e)?,
        EditorFmt::Yaml => serde_yaml::to_string(e)?,
    })
}

fn from_text(fmt: EditorFmt, s: &str) -> anyhow::Result<EditableNote> {
    Ok(match fmt {
        EditorFmt::Json => serde_json::from_str(s)?,
        EditorFmt::Toml => toml::from_str(s)?,
        EditorFmt::Yaml => serde_yaml::from_str(s)?,
    })
}

fn open_in_editor(initial: &str, fmt: EditorFmt) -> anyhow::Result<String> {
    let ext = match fmt { EditorFmt::Json => "json", EditorFmt::Toml => "toml", EditorFmt::Yaml => "yaml" };
    let mut tmp = TempBuilder::new().suffix(&format!(".{}", ext)).tempfile()?;
    tmp.write_all(initial.as_bytes())?;
    tmp.flush()?;

    let path = tmp.path().to_path_buf();

    // Select default editor based on OS: $VISUAL > $EDITOR > default
    let default_editor = if cfg!(windows) { "notepad".to_string() } else { "vi".to_string() };
    let editor_env = env::var("VISUAL").ok().or_else(|| env::var("EDITOR").ok()).unwrap_or(default_editor);

    // Allow commands with flags, e.g. "code -w"
    let mut parts = shell_words::split(&editor_env).unwrap_or_else(|_| vec![editor_env.clone()]);
    if parts.is_empty() { parts.push(editor_env); }

    let status = ProcCommand::new(&parts[0])
        .args(&parts[1..])
        .arg(&path)
        .status();

    match status {
        Ok(s) if s.success() => {
            let edited = fs::read_to_string(&path)?;
            Ok(edited)
        }
        Ok(s) => anyhow::bail!("The editor ended with status: {:?}", s.code()),
        Err(e) => anyhow::bail!("Couldn't run the editor: {}", e),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let store_path = cli
        .store
        .unwrap_or_else(|| PathBuf::from(default_store_for(cli.format)));

    let mut storage = load(&store_path, cli.format)?;

    match cli.command {
        Command::Add { title, body, mut tags } => {
            tags = normalize_tags(tags);
            let note = Note {
                id: storage.next_id(),
                title,
                body,
                tags,
                created_at: Utc::now(),
            };
            storage.notes.push(note.clone());
            save(&store_path, cli.format, &storage)?;
            println!(
                "✅ Note #{} saved to {}",
                note.id,
                store_path.display()
            );
        }
        Command::List { long } => {
            if storage.notes.is_empty() {
                println!("(No Notes yet)");
            } else {
                for n in &storage.notes {
                    println!(
                        "#{:>3}  {}  [{}]  {}",
                        n.id,
                        n.title,
                        if n.tags.is_empty() { "".to_string() } else { n.tags.join(",") },
                        n.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    if long && !n.body.is_empty() {
                        println!("    {}", n.body);
                    }
                }
            }
        }
        Command::Search { query, tags } => {
            let q = query.to_lowercase();

            let results = storage.notes.iter().filter(|n| {
                let text_match = n.title.to_lowercase().contains(&q)
                    || n.body.to_lowercase().contains(&q)
                    || n.tags.iter().any(|t| t.to_lowercase().contains(&q));

                let tags_match = if tags.is_empty() {
                    true
                } else {
                    let ntags: Vec<String> = n.tags.iter().map(|t| t.to_lowercase()).collect();
                    tags.iter().all(|t| ntags.contains(&t.to_lowercase()))
                };

                text_match && tags_match
            });

            let mut count = 0;
            for n in results {
                count += 1;
                println!(
                    "#{:>3}  {}  [{}]  {}",
                    n.id,
                    n.title,
                    if n.tags.is_empty() { "".to_string() } else { n.tags.join(",") },
                    n.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
            if count == 0 {
                if tags.is_empty() {
                    println!("No results for \"{}\"", query);
                } else {
                    println!("No results for \"{}\" with tags {:?}", query, tags);
                }
            }
        }
        Command::Remove { id } => {
            if let Some(pos) = storage.notes.iter().position(|n| n.id == id) {
                let removed = storage.notes.remove(pos);
                save(&store_path, cli.format, &storage)?;
                println!("🗑️ Note deleted #{}: {}", removed.id, removed.title);
            } else {
                println!("⚠️ Didn't find the note with id {}", id);
            }
        }
        Command::Edit { id, title, body, tags, add_tags: plus, rm_tags: minus, open_editor, editor_format } => {
            // To avoid the active mutable borrow when saving, two phases:
            // 1) Mutate and prepare data to print. 2) Save and then print.
            let mut out: Option<(u64, String, String)> = None;

            if let Some(n) = storage.notes.iter_mut().find(|n| n.id == id) {
                // 1) Edit in editor if requested
                if open_editor {
                    let initial = to_text(editor_format, &editable_from_note(n))?;
                    let edited_text = open_in_editor(&initial, editor_format)?;
                    let edited = from_text(editor_format, &edited_text)?;

                    if let Some(t) = edited.title { n.title = t; }
                    if let Some(b) = edited.body { n.body = b; }
                    if let Some(ts) = edited.tags { n.tags = normalize_tags(ts); }
                }

                // 2) Flags from command line (applied after editor)
                if let Some(t) = title { n.title = t; }
                if let Some(b) = body { n.body = b; }
                if let Some(ts) = tags { n.tags = normalize_tags(ts); }
                if !plus.is_empty() { add_tags(&mut n.tags, plus); }
                if !minus.is_empty() { remove_tags(&mut n.tags, minus); }

                let id_out = n.id;
                let title_out = n.title.clone();
                let tags_out = if n.tags.is_empty() { String::new() } else { n.tags.join(",") };
                out = Some((id_out, title_out, tags_out));
            } else {
                println!("⚠️ Didn't find the note with id {}", id);
            }

            if let Some((id_out, title_out, tags_out)) = out {
                save(&store_path, cli.format, &storage)?;
                println!("✏️ Note #{} updated: {}  [{}]", id_out, title_out, tags_out);
            }
        }
    }

    Ok(())
}