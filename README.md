# Refactoring in Rust

Progressive refactoring exercises using only the Rust standard library. Each exercise has five levels covering readability, data modeling, ownership, and behavior-preserving changes, with regression tests in the final solution.

## Exercises

| Package | Exercise | Main topics |
| --- | --- | --- |
| `task_1` | [Error-log reporting](src/task_1/README.md) | Named records, separate aggregation and formatting, borrowed service names, single-pass counting with `HashMap` |
| `task_2` | [Retry configuration parser](src/task_2/README.md) | Configuration types, typed errors, validation priority, parsing borrowed input without temporary strings |
| `task_3` | [Job dispatch batch](src/task_3/README.md) | State modeling, ordered selection, deduplication with `HashSet`, borrowed candidates |
| `task_4` | [Bounded FIFO cache](src/task_4/README.md) | Encapsulation, typed errors, FIFO eviction with `VecDeque`, borrowed lookups |
| `task_5` | [HTTP retry planner](src/task_5/README.md) | Validation order, action enums, separate decisions and formatting, borrowed requests and owned output |

Open an exercise's README for its behavior contract, level-by-level goals, regression scenarios, and complexity analysis.

## Project structure

The root `Cargo.toml` defines a workspace with members matching `src/task_*`. Each task is an independent package with this layout (shown for `task_1`):

```text
src/task_1/
  Cargo.toml
  README.md
  task_1_start.rs
  task_1_final.rs
```

Both Rust files have their own `fn main()`. The `start` binary preserves the original implementation; the `final` binary contains the refactored solution and its tests.

## Running the code

Use a Rust toolchain with edition 2024 support, Cargo, rustfmt, and Clippy. Run commands from the project root; no external services or crate dependencies are required.

```sh
# Initial version (default)
cargo run -p task_1

# Explicitly run the initial version
cargo run -p task_1 --bin start

# Refactored solution
cargo run -p task_1 --bin final

# Run another exercise
cargo run -p task_5 --bin final
```

Replace `task_1` with any package listed above. The default binary is `start` for every package. The exercise READMEs also include standalone setup examples; creating a separate project is unnecessary when using this workspace.

## Verification

```sh
# Compile all tasks and both versions
cargo check --workspace

# Run all regression tests
cargo test --workspace

# Test one solution
cargo test -p task_5 --bin final

# Check formatting
cargo fmt --all --check

# Lint all refactored solutions
cargo clippy --workspace --bin final -- -D warnings
```

The starting implementations intentionally retain Clippy warnings. Select `--bin final` when linting solutions with warnings treated as errors.

The workspace formatting check also includes the starting implementations and currently reports formatting differences in `task_4_start.rs` and `task_5_start.rs`.

## Adding another task

For example, to add `task_6`:

1. Create the `src/task_6` directory.
2. Add `task_6_start.rs` and `task_6_final.rs`, each with its own `fn main()`.
3. Add a `README.md` describing the behavior contract and refactoring levels.
4. Add a `Cargo.toml` file with a unique package name and paths matching the new files:

```toml
[package]
name = "task_6"
version = "0.1.0"
edition = "2024"
default-run = "start"

[[bin]]
name = "start"
path = "task_6_start.rs"

[[bin]]
name = "final"
path = "task_6_final.rs"
```

Then run `cargo run -p task_6` or `cargo run -p task_6 --bin final`, run the workspace checks above, and add the exercise to the table in this README. Commit the updated `Cargo.lock` along with the new task.

The root `Cargo.toml` does not need to be updated: the workspace automatically includes `src/task_*` directories that have their own `Cargo.toml`.
