use clap::Parser;
use regex::{Regex, RegexBuilder};
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use ignore::{overrides::OverrideBuilder, WalkBuilder};

/// A simple grep-like tool that searches for regex patterns in files and directories.
#[derive(Parser, Debug)]
#[command(name = "mini-grep", version, about = "Search for regex patterns in files and directories")]
struct Args {
    /// Ignore case (case-insensitive)
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Show line number
    #[arg(short = 'n', long = "line-number")]
    line_number: bool,

    /// Search directories recursively
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,

    /// Match whole words only (\b...\b)
    #[arg(short = 'w', long = "word")]
    word: bool,

    /// Disable ANSI colors (by default, matches are highlighted in red)
    #[arg(long = "no-color")]
    no_color: bool,

    /// Maximum total matches (stop when reached)
    #[arg(short = 'm', long = "max-count")]
    max_count: Option<usize>,

    /// Include hidden files (by default they are ignored)
    #[arg(long = "hidden")]
    hidden: bool,

    /// Do not respect .gitignore/.ignore (by default they are respected)
    #[arg(long = "no-ignore")]
    no_ignore: bool,

    /// Glob patterns to filter files (e.g. --glob "**/*.rs"), can be repeated
    #[arg(long = "glob", value_name = "GLOB")]
    globs: Vec<String>,

    /// Include binary files (by default they are skipped if they seem binary)
    #[arg(long = "binary")]
    include_binary: bool,

    /// Search pattern (regex)
    #[arg(value_name = "PATTERN")]
    pattern: String,

    /// Path(s) to files or directories (default: .)
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<PathBuf>,
}

#[derive(Debug)]
enum MiniGrepError {
    Io(io::Error),
    Regex(regex::Error),
    Ignore(ignore::Error),
}

impl Display for MiniGrepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MiniGrepError::Io(e) => write!(f, "I/O: {e}"),
            MiniGrepError::Regex(e) => write!(f, "Regex: {e}"),
            MiniGrepError::Ignore(e) => write!(f, "Ignore/.gitignore: {e}"),
        }
    }
}
impl From<io::Error> for MiniGrepError { fn from(e: io::Error) -> Self { Self::Io(e) } }
impl From<regex::Error> for MiniGrepError { fn from(e: regex::Error) -> Self { Self::Regex(e) } }
impl From<ignore::Error> for MiniGrepError { fn from(e: ignore::Error) -> Self { Self::Ignore(e) } }

type Result<T> = std::result::Result<T, MiniGrepError>;

struct Options {
    line_number: bool,
    color: bool,
    max_count: Option<usize>,
    skip_binary: bool,
}

fn main() {
    match run() {
        Ok(true) => std::process::exit(0),
        Ok(false) => std::process::exit(1),
        Err(e) => { eprintln!("mini-grep: {e}"); std::process::exit(2); }
    }
}

fn run() -> Result<bool> {
    let args = Args::parse();

    let pattern = if args.word {
        format!(r"\b(?:{})\b", args.pattern)
    } else {
        args.pattern.clone()
    };

    let re = RegexBuilder::new(&pattern)
        .case_insensitive(args.ignore_case)
        .build()?;

    let opts = Options {
        line_number: args.line_number,
        color: !args.no_color,
        max_count: args.max_count,
        skip_binary: !args.include_binary,
    };

    let mut found_any = false;
    let mut emitted = 0usize;

    for path in &args.paths {
        if path.is_file() {
            let f = search_file(path, &re, &opts, &mut emitted)?;
            found_any = found_any || f;
            if stop_now(&opts, emitted) { break; }
        } else if path.is_dir() {
            if args.recursive {
                // Walker respects .gitignore/.ignore by default
                let mut builder = WalkBuilder::new(path);
                builder.hidden(!args.hidden);
                if args.no_ignore {
                    builder
                        .git_ignore(false)
                        .git_global(false)
                        .git_exclude(false)
                        .ignore(false)
                        .parents(false);
                }
                if !args.globs.is_empty() {
                    let mut ob = OverrideBuilder::new(path);
                    for g in &args.globs { ob.add(g)?; }
                    builder.overrides(ob.build()?);
                }
                for entry in builder.build() {
                    match entry {
                        Ok(e) => {
                            if e.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                                let file_path = e.into_path();
                                let f = search_file(&file_path, &re, &opts, &mut emitted)?;
                                found_any = found_any || f;
                                if stop_now(&opts, emitted) { break; }
                            }
                        }
                        Err(err) => eprintln!("mini-grep: error: {err}"),
                    }
                }
            } else {
                eprintln!(
                    "mini-grep: {} is a directory (use -r to search recursively)",
                    path.display()
                );
            }
        } else {
            eprintln!("mini-grep: not exists {}", path.display());
        }
    }

    Ok(found_any)
}

fn stop_now(opts: &Options, emitted: usize) -> bool {
    if let Some(m) = opts.max_count { emitted >= m } else { false }
}

fn search_file(path: &Path, re: &Regex, opts: &Options, emitted: &mut usize) -> Result<bool> {
    if opts.skip_binary && is_probably_binary(path)? {
        return Ok(false);
    }

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut buf = Vec::<u8>::new();
    let mut line_no: usize = 0;
    let mut found = false;

    loop {
        buf.clear();
        let n = reader.read_until(b'\n', &mut buf)?;
        if n == 0 { break; }
        line_no += 1;

        let line = String::from_utf8_lossy(&buf);
        let line_str = line.trim_end_matches(&['\n', '\r'][..]);

        if let Some(mat) = re.find(line_str) {
            found = true;
            let column = 1 + line_str[..mat.start()].chars().count();
            let highlighted = if opts.color { highlight_matches(line_str, re) } else { line_str.to_owned() };

            if opts.line_number {
                println!("{}:{}:{}: {}", path.display(), line_no, column, highlighted);
            } else {
                println!("{}:{}: {}", path.display(), column, highlighted);
            }

            *emitted += 1;
            if stop_now(opts, *emitted) { break; }
        }
    }

    Ok(found)
}

/// Simple heuristic: if the first bytes contain NUL, we treat it as binary.
fn is_probably_binary(path: &Path) -> Result<bool> {
    let mut f = File::open(path)?;
    let mut buf = [0u8; 1024];
    let n = f.read(&mut buf)?;
    Ok(buf[..n].contains(&0))
}

/// Highlights all matches with ANSI sequences (red) and resets color at the end.
fn highlight_matches(s: &str, re: &Regex) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    let mut last = 0usize;
    for m in re.find_iter(s) {
        out.push_str(&s[last..m.start()]);
        out.push_str("\x1b[31m"); // red
        out.push_str(&s[m.start()..m.end()]);
        out.push_str("\x1b[0m"); // reset
        last = m.end();
    }
    out.push_str(&s[last..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_boundary_wraps_pattern() {
        let pat = "rust|go";
        let wrapped = format!(r"\b(?:{})\b", pat);
        assert_eq!(wrapped, r"\b(?:rust|go)\b");
    }

    #[test]
    fn highlight_inserts_ansi() {
        let re = Regex::new("rust").unwrap();
        let s = "i love rust";
        let h = highlight_matches(s, &re);
        assert!(h.contains("\x1b[31m"));
    }
}
