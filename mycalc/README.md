# 🧮 mycalc — Simple CLI Calculator

A small, friendly command‑line calculator built with the `clap` crate. It supports basic arithmetic via subcommands and a global precision flag for formatted output.

## ✨ Features

- ➕ ➖ ✖️ ➗: Subcommands for add, sub, mul, div
- 🎯 Global precision: `-p, --precision <N>` (default: 2)
- 🚫 Safe division: error on division by zero (non‑zero exit)
- 🧰 Clean structure: logic in `lib.rs`, CLI in `main.rs`

## 🧱 Requirements

- Rust and Cargo (stable). Install via `rustup` if needed.

## 🔧 Build

From the repo root:

```
cargo build -p mycalc
```

Or from the project directory:

```
cargo build
```

## 🚀 Run

From the repo root:

```
cargo run -p mycalc -- <ARGS>
```

Or from the project directory:

```
cargo run -- <ARGS>
```

## 📦 Install (optional)

Install the binary locally so you can run `mycalc` directly:

```
cargo install --path .
```

From the repo root, you can also run:

```
cargo install --path mycalc
```

## 📝 Usage

```
mycalc [--precision <N>] <COMMAND> <NUM> <NUM> [NUM ...]

Commands:
  add   Add all numbers
  sub   Subtract left‑associative (e.g., 10 3 2 => (10 - 3 - 2))
  mul   Multiply all numbers
  div   Divide left‑associative; errors on division by zero

Global Options:
  -p, --precision <N>   Decimal places to print (default: 2)
  -h, --help            Print help
  -V, --version         Print version
```

## 🔍 Examples

- Add numbers:
  - `mycalc add 1 2 3` → `6.00`
- Subtract (left‑associative):
  - `mycalc sub 10 3 2` → `(10 - 3 - 2)` → `5.00`
- Multiply:
  - `mycalc mul 2 3 4` → `24.00`
- Divide (left‑associative):
  - `mycalc div 20 2 5` → `(20 / 2 / 5)` → `2.00`
- Increase precision:
  - `mycalc -p 4 div 7 3` → `2.3333`

Division by zero prints an error to stderr and exits with code `1`:

```
$ mycalc div 10 0
Error: division by zero
$ echo $?
1
```

## 🧪 Tests

Run the unit and CLI tests:

```
cargo test -p mycalc
```

## 📂 Notes

- At least two numbers are required for each operation.
- Subtraction and division are left‑associative.
- Precision applies to all printed results.

## 📄 License

MIT (see `Cargo.toml`).

