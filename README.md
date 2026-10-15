# Refactoring in Rust

Each task has its own directory in `src/`, an initial version named `start`, and a final version named `final`.

```text
src/
  task_1/
    Cargo.toml
    task_1_start.rs
    task_1_final.rs
  task_2/
    Cargo.toml
    task_2_start.rs
    task_2_final.rs
```

## Running the code

Run commands from the project root.

```sh
# Initial version (default)
cargo run -p task_1

# Explicitly run the initial version
cargo run -p task_1 --bin start

# Final version
cargo run -p task_1 --bin final

# Check all tasks
cargo check --workspace
```

The `task_1_final.rs` file implements Levels 1–5: a named data model, separate error counting and report formatting, borrowed input without cloning service names, and single-pass aggregation using a `HashMap`, followed by sorting the results. Regression tests cover the behavior contract and a larger input. Both versions have their own `fn main()`.

```sh
# Tests and formatting
cargo test --workspace
cargo fmt --all --check

# Clippy for the final version (the initial version has intentional lint violations)
cargo clippy -p task_1 --bin final -- -D warnings
```

## Adding another task

1. Create the `src/task_2` directory.
2. Add `task_2_start.rs` and `task_2_final.rs`, each with its own `fn main()`.
3. Add a `Cargo.toml` file to this directory:

```toml
[package]
name = "task_2"
version = "0.1.0"
edition = "2024"
default-run = "start"

[[bin]]
name = "start"
path = "task_2_start.rs"

[[bin]]
name = "final"
path = "task_2_final.rs"
```

Then run `cargo run -p task_2` or `cargo run -p task_2 --bin final`.
The root `Cargo.toml` does not need to be updated: the workspace automatically includes `src/task_*` directories that have their own `Cargo.toml`.
