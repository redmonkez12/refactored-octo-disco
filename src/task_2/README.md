# Rust Refactoring Challenge 02: Retry Configuration Parser

## Summary

Refactor a working configuration parser through five successive levels while preserving its behavior.

An HTTP client reads retry settings from text lines. The current parser uses an anonymous tuple, string-based errors, and unnecessary heap allocations. Your goal is to introduce a clear data model, typed errors, and borrowed input, then verify the result with regression tests.

**Estimated time:** 50 minutes. **Dependencies:** Rust standard library only.

| Level | Focus | Estimated time |
| --- | --- | --- |
| 1 | Readability and control flow | 5 minutes |
| 2 | A named configuration type | 8 minutes |
| 3 | Typed errors and error presentation | 12 minutes |
| 4 | Borrowing and allocation removal | 12 minutes |
| 5 | Regression tests and complexity analysis | 13 minutes |

Work through the levels in order. Function signatures and internal types may change as required, but accepted inputs, rejected inputs, error priority, and console output must stay the same. `task_2_start.rs` contains the starting code, and `task_2_final.rs` implements Levels 1–5.

## Setup

Create a binary project:

```bash
cargo new retry-config-refactoring
cd retry-config-refactoring
```

Replace `src/main.rs` with the starting code below, then run `cargo run`.

## Starting code

```rust
fn read_settings(v: Vec<String>) -> Result<(u32, u64), String> {
    let mut a = 3;
    let mut d = 1000;
    let mut seen_a = false;
    let mut seen_d = false;

    for i in 0..v.len() {
        let raw = v[i].clone();
        let parts: Vec<&str> = raw.split('=').collect();

        if parts.len() != 2 {
            return Err("invalid_format".to_string());
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        if key == "attempts" {
            if seen_a {
                return Err("duplicate_key".to_string());
            }
            seen_a = true;

            match value.parse::<u32>() {
                Ok(number) => {
                    if number < 1 || number > 10 {
                        return Err("invalid_attempts".to_string());
                    }
                    a = number;
                }
                Err(_) => return Err("invalid_attempts".to_string()),
            }
        } else if key == "delay_ms" {
            if seen_d {
                return Err("duplicate_key".to_string());
            }
            seen_d = true;

            match value.parse::<u64>() {
                Ok(number) => {
                    if number > 60_000 {
                        return Err("invalid_delay".to_string());
                    }
                    d = number;
                }
                Err(_) => return Err("invalid_delay".to_string()),
            }
        } else {
            return Err("unknown_key".to_string());
        }
    }

    Ok((a, d))
}

fn main() {
    let input = vec![
        " attempts = 4 ".to_string(),
        "delay_ms=250".to_string(),
    ];

    match read_settings(input) {
        Ok((attempts, delay)) => {
            println!("attempts={attempts};delay_ms={delay}");
        }
        Err(error) => println!("error={error}"),
    }
}
```

Expected output:

```text
attempts=4;delay_ms=250
```

## Behavior contract

### Configuration values

| Key | Rust value type | Default | Allowed range | Meaning |
| --- | --- | --- | --- | --- |
| `attempts` | `u32` | `3` | `1..=10` | Total attempts, including the first attempt |
| `delay_ms` | `u64` | `1000` | `0..=60000` | Delay between attempts, in milliseconds |

Each setting is optional. Empty input returns both defaults.

### Input rules

- Every line must contain exactly one `=` character.
- An empty line, a line without `=`, or a line with multiple `=` characters is invalid.
- Apply Rust's `trim()` to both the key and value.
- Key matching is case-sensitive. Only `attempts` and `delay_ms` are recognized.
- A known key may appear at most once, including when surrounding whitespace differs.
- Parse values with the semantics of `parse::<u32>()` and `parse::<u64>()`.
- A leading `+` is accepted. Negative values, non-numeric text, and integer overflow are rejected.
- The parser must return an error for invalid input rather than panic.

### Error codes and priority

| Error code | Condition |
| --- | --- |
| `invalid_format` | A line does not contain exactly one `=` |
| `unknown_key` | The trimmed key is not recognized, including an empty key |
| `duplicate_key` | A recognized key has already appeared |
| `invalid_attempts` | The attempts value cannot be parsed or is outside `1..=10` |
| `invalid_delay` | The delay value cannot be parsed or is outside `0..=60000` |

Process lines in input order and stop at the first error. Within each line, preserve this order:

1. Validate the line format.
2. Identify the key.
3. Reject a duplicate known key.
4. Parse and validate the value.

For example, `attempts=4` followed by `attempts=nope` produces `duplicate_key`, because duplicate detection happens before value parsing. A duplicate line containing two `=` characters produces `invalid_format`, because format validation happens first.

### Console output

- Success: `attempts={attempts};delay_ms={delay}`.
- Failure: `error={code}`.
- Print one line and preserve the exact code spellings.

## Level 1: Readability

**Estimated time:** 5 minutes.

- [x] Replace unclear variable names with meaningful names.
- [x] Make the parsing and validation flow easy to follow.
- [x] Simplify unnecessary branching where it helps readability.

**Done when:** another developer can identify the configuration values and understand each validation step without decoding variable names. Accepted inputs, output, and error priority remain unchanged.

The Level 1 solution uses meaningful names, iterates directly over input lines, and dispatches on the key with `match`. Parsing uses `?` to return errors before range validation, keeping each validation step visible without nested success branches. Format validation still runs before key lookup, duplicate detection, and value validation. At Level 1, the tuple result, string errors, input ownership, and temporary allocations were retained for later levels.

From the workspace root, run `cargo run -p task_2 --bin final`, `cargo test -p task_2 --bin final`, `cargo fmt --all --check`, and `cargo clippy -p task_2 --bin final -- -D warnings` to check this level. The `start` binary retains its intentional lint violations.

## Level 2: A named configuration type

**Estimated time:** 8 minutes.

- [x] Introduce a `RetryConfig` struct with named fields.
- [x] Replace the return tuple with `RetryConfig`.
- [x] Update the caller to use the named fields.
- [x] Preserve the defaults and all validation rules.

**Done when:** the attempts count and delay have explicit names, and callers no longer depend on tuple positions.

The Level 2 solution returns `RetryConfig` with `attempts: u32` and `delay_ms: u64`. `main` reads these named fields instead of destructuring a tuple. Defaults, validation rules, error priority, and console output remain unchanged.

## Level 3: Typed errors

**Estimated time:** 12 minutes.

- [x] Introduce a `ConfigError` enum with variants corresponding to the five existing error codes.
- [x] Make the parser return `Result<RetryConfig, ConfigError>`.
- [x] Separate error-to-text conversion from parsing.
- [x] Keep the existing `error=...` console output.

**Done when:** the parser no longer creates error `String` values. Callers can distinguish failures by enum variant, and the presentation layer preserves the original error codes.

The Level 3 solution returns `ConfigError` variants from the parser. A separate `Display` implementation maps each variant to its original error code without creating an error `String`. `main` continues to print `error={error}`, preserving the existing console format. Validation rules and error priority remain unchanged.

## Level 4: Borrowing and allocations

**Estimated time:** 12 minutes.

- [x] Change the parser input to `&[String]`.
- [x] Remove the clone of each input line.
- [x] Stop creating owned copies of keys and values.
- [x] Remove the temporary `Vec` used to split a line.
- [x] Keep the rule requiring exactly one `=`.
- [x] Demonstrate that the caller can still use the input after parsing.

**Target API:**

```rust
fn read_settings(input: &[String]) -> Result<RetryConfig, ConfigError>
```

This is a function signature, not a complete implementation.

**Done when:** parsing does not allocate owned text copies or temporary collections, and it neither consumes nor modifies the input. Constructing the input in `main` and formatting console output are outside this allocation requirement.

The Level 4 solution borrows the input as `&[String]` and uses `split_once` and `trim` to borrow keys and values directly from each line. It checks for an additional `=` before key lookup, preserving both the format rule and error priority without temporary collections or owned text copies. After parsing, `main` asserts that the original input is still available and unchanged without adding console output. Level 5 also verifies input reuse after both success and failure.

## Level 5: Regression tests and complexity

**Estimated time:** 13 minutes.

- [x] Test empty input and partial configurations.
- [x] Test both accepted range boundaries and rejected values.
- [x] Test every error variant and error precedence.
- [x] Verify whitespace handling, case sensitivity, and leading `+`.
- [x] Cover negative values, numeric overflow, and malformed lines.
- [x] Verify that parsing leaves the input available to its caller.
- [x] Compare the time and auxiliary space complexity before and after refactoring.

**Done when:** tests inspect configuration values and specific error variants. The complexity explanation accounts for input byte length as well as line count, and distinguishes borrowed input storage from temporary allocations.

The Level 5 tests compare `RetryConfig` values and individual `ConfigError` variants using derived `Debug`, `PartialEq`, and `Eq` implementations. They cover defaults, both key orders, inclusive range boundaries, malformed numbers and lines, ASCII and Unicode whitespace, leading `+`, duplicate detection before value parsing, format validation before key lookup or duplicate detection, and the first error across lines. Separate tests verify all five `Display` codes and reuse of unchanged input after success or failure.

### Complexity comparison

Let `n` be the number of input lines, `B` their total byte length, and `L` the longest line. Both versions have a worst-case time bound of `O(n + B)`: line iteration has a cost, and string operations must examine bytes. Counting only lines would miss the cost of arbitrarily long keys, values, or whitespace. For nonempty text lines this is commonly simplified to `O(B)`.

The refactored parser returns immediately on the first error, so its time is at most `O(P)`, where `P` is the byte length of the input prefix through the last examined line (`O(1)` for empty input or an empty first line). `split_once`, `contains`, `trim`, and integer parsing each scan at most part of a line; a constant number of passes remains linear.

| Resource | Starting version | Refactored version |
| --- | --- | --- |
| Worst-case parsing time | `O(n + B)` | `O(n + B)`, with early return |
| Peak auxiliary space, excluding input | `O(L)` | `O(1)` |
| Temporary allocations | Cloned line, split collection, owned key and value, and an error string on failure | None inside the parser |

The starting version holds temporary copies for only one line at a time. Its split collection can contain `O(L)` slices when a malformed line has many separators, so peak auxiliary space is `O(L)`, rather than the size of all input. The existing `Vec<String>` and its text occupy `O(n + B)` storage in either version and are excluded from auxiliary space. The starting version consumes and drops that input; the refactored version only borrows it. Test fixtures and console formatting are outside the parser's allocation requirement.

Removing allocations reduces copying, heap operations, and peak temporary memory. It does not remove the need to read the text, so the asymptotic time bound stays linear in input bytes.

## Verification examples

The input column lists the text lines supplied to the parser.

| Input lines | Expected console output |
| --- | --- |
| `[]` | `attempts=3;delay_ms=1000` |
| `["delay_ms=0"]` | `attempts=3;delay_ms=0` |
| `["attempts=10", "delay_ms=60000"]` | `attempts=10;delay_ms=60000` |
| `["attempts=+4"]` | `attempts=4;delay_ms=1000` |
| `["attempts=0"]` | `error=invalid_attempts` |
| `["delay_ms=60001"]` | `error=invalid_delay` |
| `["attempts=4", "attempts=nope"]` | `error=duplicate_key` |
| `["attempts=11", "other=1"]` | `error=invalid_attempts` |
| `["other=1"]` | `error=unknown_key` |
| `["attempts=1=2"]` | `error=invalid_format` |
| `[""]` | `error=invalid_format` |

Also check these precedence cases:

| Input lines | Expected error code |
| --- | --- |
| `["other=1=2"]` | `invalid_format` |
| `["attempts=3", "attempts=4=5"]` | `invalid_format` |
| `["attempts=3", " attempts = 4"]` | `duplicate_key` |
| `["Attempts=3"]` | `unknown_key` |
| `["=1"]` | `unknown_key` |
| `["attempts="]` | `invalid_attempts` |
| `["delay_ms=-1"]` | `invalid_delay` |

## Final checks

Run from the workspace root:

```bash
cargo test --workspace
cargo fmt --all --check
cargo clippy -p task_2 --bin final -- -D warnings
cargo run -p task_2 --bin final
```

The `start` binaries retain intentional lint violations and are not included in the Clippy check for the final parser.

## What to submit

- Your refactored code and the highest level completed.
- Regression tests for the behavior contract and error priority.
- A short explanation of the data model and error representation.
- A comparison of time complexity, auxiliary space, and temporary allocations.

**Interview question:** Why can removing allocations leave a program's asymptotic time complexity unchanged?

## Completion checklist

- [x] All five levels are complete.
- [x] The original behavior contract is preserved.
- [x] The example output and error codes remain unchanged.
- [x] The parser uses borrowed input and typed errors.
- [x] Regression tests and final checks pass.
- [x] The complexity comparison is written in your own words.

