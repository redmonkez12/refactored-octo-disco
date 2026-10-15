# Rust Refactoring Practice — Challenge 01

## Overview

Refactor an existing error-log reporting function through five successive levels. Each level builds on the previous one.

**Time:** 40–60 minutes. **Dependencies:** Rust standard library only.

Keep the observable behavior unchanged. Function signatures and internal types may change when a level requires it. The starting implementation is intentionally awkward; `task_1_final.rs` implements Levels 1–5.

## Behavior contract

Each input record is a `(service, level)` pair. The report must:

- Count only records with the exact, case-sensitive level `"error"`.
- Ignore all other levels, including unknown values.
- Include each service with at least one error exactly once.
- Preserve service names exactly, without trimming or normalization.
- Return a `Vec<String>` containing entries formatted as `"{service}: {count}"`.
- Sort the entries by service name using Rust's standard ascending string order.
- Return an empty vector for empty input or input containing no errors.

## Setup

Create a binary project:

```bash
cargo new rust-refactoring-01
cd rust-refactoring-01
```

Copy the starting code below into `src/main.rs`, then run `cargo run`.

## Starting code

```rust
fn report(a: Vec<(String, String)>) -> Vec<String> {
    let mut b = Vec::new();

    for i in 0..a.len() {
        if a[i].1 == "error" {
            let mut found = false;

            for j in 0..b.len() {
                if b[j] == a[i].0 {
                    found = true;
                }
            }

            if found == false {
                b.push(a[i].0.clone());
            }
        }
    }

    b.sort();

    let mut c = Vec::new();

    for i in 0..b.len() {
        let mut n = 0usize;

        for j in 0..a.len() {
            if a[j].0 == b[i] {
                if a[j].1 == "error" {
                    n += 1;
                }
            }
        }

        c.push(format!("{}: {}", b[i], n));
    }

    c
}

fn main() {
    let logs = vec![
        ("payments".into(), "error".into()),
        ("api".into(), "info".into()),
        ("api".into(), "error".into()),
        ("payments".into(), "error".into()),
    ];

    println!("{:?}", report(logs));
}
```

Expected output:

```text
["api: 1", "payments: 2"]
```

## Level 1 — Readability

**Time:** 5 minutes.

- [x] Replace single-letter variable names with meaningful names.
- [x] Simplify unnecessary nesting and boolean comparisons.
- [x] Make the purpose of each loop clear.

**Done when:** the output is unchanged and another developer can understand the function without decoding its variable names. Keep the existing algorithm for now.

## Level 2 — Named data model

**Time:** 10 minutes.

- [x] Replace the input tuples with a `LogRecord` struct.
- [x] Give the struct named fields for the service and log level.
- [x] Update the example input and processing code.

**Done when:** processing uses named fields instead of `.0` and `.1`. Unknown levels are still ignored, and service names remain unchanged.

## Level 3 — Separate responsibilities

**Time:** 10 minutes.

- [x] Separate error counting from report formatting.
- [x] Return structured counts from the aggregation logic.
- [x] Keep console output in `main`.

**Done when:** counts can be tested without parsing formatted strings, and formatting can be checked independently. The final report still follows the original contract.

The Level 3 solution introduced `count_errors` to return sorted `ServiceErrorCount` values, `format_report` to turn those values into strings, and `report` to compose both steps. Only `main` prints output. Tests cover counting and formatting independently as well as the report's behavior contract.

At Level 3, input was consumed as `Vec<LogRecord>`, and aggregation owned one cloned service name per distinct service with errors. Repeated scans were retained: both before and after Level 3, counting took O(n + n × s), sorting took O(s log s), and formatting allocated one output string per service. Here `n` is the number of records and `s` the number of distinct services with errors; these operation counts treat string comparisons as constant time. Level 3 aggregation used O(s) additional entries plus the copied service-name bytes, and formatting takes time and space proportional to the total output bytes.

From the workspace root, run `cargo test -p task_1 --bin final` and `cargo clippy -p task_1 --bin final -- -D warnings` to check the solution. The intentionally awkward `start` binary has existing Clippy warnings.

## Level 4 — Ownership and borrowing

**Time:** 10 minutes.

- [x] Make the report accept borrowed input as `&[LogRecord]`.
- [x] Avoid cloning service names while reading and aggregating records.
- [x] Allocate owned strings for the final report.
- [x] Demonstrate that the caller can still use the input after generating a report.

**Done when:** the input is neither consumed nor modified, no input clone is required at the call site, aggregation does not copy service names into new owned strings, and the report still returns `Vec<String>`.

The Level 4 solution introduced input borrowing via `&[LogRecord]` and borrowed service names as `&'a str` in `ServiceErrorCount<'a>`. Counts cannot outlive the input they borrow from; `format_report` creates owned output strings that can outlive that input. `main` uses `logs` after generating the report, and tests verify borrowed name storage, unchanged and reusable input, and output surviving the input's scope. At Level 4, the time complexity was unchanged from Level 3, while aggregation used O(s) additional entries without copying service-name bytes. Formatting still allocates one string per service and takes time and space proportional to the total output bytes.

## Level 5 — Performance and regression tests

**Time:** 15 minutes.

- [x] Aggregate counts in one pass over the input records.
- [x] Remove repeated scans of all records for each service.
- [x] Preserve the required output ordering.
- [x] Add regression tests covering the scenarios below.
- [x] Explain the time and space complexity before and after refactoring, including sorting.

**Done when:** aggregation visits each input record once, all scenarios pass, and your chosen data structure satisfies both borrowing and ordering requirements.

The Level 5 solution counts errors in a single `for log in logs` pass using `HashMap<&str, usize>`. Keys borrow service names from the input, so aggregation neither clones names nor consumes or modifies records. Map entries become structured counts, which are sorted by the original service name with `sort_unstable_by` before formatting. Sorting removes any dependence on the map's iteration order; stability is unnecessary because each service has exactly one entry.

Let `n` be the number of input records, `s` the number of distinct services with errors, and `b` the total bytes of the formatted output. Excluding input storage and treating string hashing and comparisons as constant time:

| Stage | Before (Level 4) | After (Level 5) |
| --- | --- | --- |
| Aggregation | O(n + n × s), from service searches and repeated record scans | Expected O(n), from one pass with map entry updates |
| Sorting | O(s log s) | O(s log s) |
| Formatting | O(s + b), with one owned output string per service | O(s + b), with one owned output string per service |
| Total time | O(n + n × s + s log s + b) | Expected O(n + s log s + b) |
| Additional aggregation space | O(s), borrowing service names | O(s), borrowing service names |
| Owned report space | O(s + b) | O(s + b) |

The largest improvement comes from removing the repeated scans: when every record has a distinct service with an error (`s = n`), the previous quadratic work becomes expected O(n log n), including sorting. The tradeoff is hash-table overhead and expected rather than guaranteed constant-time lookups. Pathological hash collisions can make aggregation O(n + n × s). With variable-length names, hashing also reads the service-name bytes of error records, and comparisons depend on name lengths; the output-byte cost is included separately above.

Regression coverage includes all scenarios below, empty and Unicode service names, whitespace and separator preservation, borrowed name storage, reusable input, and owned output. An independent scan-based reference checks every sequence of up to three records drawn from ten representative records (1,111 inputs). A larger case checks 50,001 records across 10,000 services with varying error counts, interleaved occurrences, reverse service order, and a service without errors. These tests use exact output assertions without machine-dependent timing limits.

## Verification scenarios

The input column uses `(service, level)` shorthand. After Level 2, construct the corresponding `LogRecord` values.

| Scenario | Input | Expected report |
| --- | --- | --- |
| Empty input | `[]` | `[]` |
| No errors | `[("api", "info"), ("worker", "warn")]` | `[]` |
| Repeated errors | `[("api", "error"), ("api", "error")]` | `["api: 2"]` |
| Reverse input order | `[("worker", "error"), ("api", "error")]` | `["api: 1", "worker: 1"]` |
| Mixed levels | `[("api", "info"), ("api", "error"), ("api", "debug")]` | `["api: 1"]` |
| Case-sensitive levels | `[("api", "ERROR"), ("worker", "error")]` | `["worker: 1"]` |
| Unknown level | `[("api", "unknown"), ("api", "error")]` | `["api: 1"]` |
| Exact service names | `[("api", "error"), ("API", "error")]` | `["API: 1", "api: 1"]` |

Run these checks from the workspace root:

```bash
cargo test --workspace
cargo fmt --all --check
cargo clippy -p task_1 --bin final -- -D warnings
```

Clippy targets the final solution; the intentionally awkward `start` binary retains its original warnings. Adding `--tests` also selects the `start` test target and reports those warnings.

## What to submit

- Your refactored code and the highest level completed.
- Regression tests covering the behavior contract.
- A brief explanation of your ownership and data-structure choices.
- A complexity comparison using `n` for the number of input records and `s` for the number of distinct services with errors. Include sorting and output allocation.

**Interview question:** Which change produced the biggest practical improvement, and what tradeoff did it introduce?

You can submit Level 1 first and work through the remaining levels with feedback.

