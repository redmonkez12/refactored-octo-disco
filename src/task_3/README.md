# Rust Refactoring Exercise 03: Job Dispatch Batch

**Estimated time:** 50 minutes\
**Dependencies:** Rust standard library only

Practice reading, refactoring, and defending existing Rust code through five successive levels. This is a standalone exercise, separate from `stack-lab`.

This original exercise is inspired by a progressive interview format. It is not an actual or confidential Better Stack interview assignment.

## Backend context

A background worker receives job candidates from several sources. It needs to select a limited batch of jobs that are ready to run. The same job can appear more than once, but it must only be included once in the batch.

The starting implementation works, but it copies data, uses anonymous tuples, and repeatedly scans previously selected jobs for duplicates.

## Behavioral contract

Preserve the following behavior throughout all five levels:

- Each input record represents `(job_id, queue, state)`.
- Only the exact state `"ready"` is eligible. All other states are skipped.
- Uniqueness is based on `job_id` alone, across all queues.
- The **first eligible occurrence** of an ID wins. Its queue is used in the output.
- An ineligible occurrence does not prevent a later eligible occurrence of the same ID from being selected.
- Results follow the input order of the selected occurrences. Do not sort them.
- `limit` caps the number of selected unique jobs, not the number of inspected records.
- Empty input and `limit == 0` both produce an empty vector.
- IDs and queue names are opaque strings. Do not trim or normalize them. They are case-sensitive and may be empty.
- No errors are returned. An unknown state simply makes a record ineligible.
- The public output is a `Vec<String>` with each entry formatted as `job_id@queue`.

The input contains candidates for this batch, not a history of state transitions. A later record does not cancel a job selected earlier.

## Local setup

Create a standalone project:

```bash
cargo new job-dispatch-batch
cd job-dispatch-batch
```

Replace `src/main.rs` with the starting code below. No extra dependencies or infrastructure are needed.

## Starting code

```rust
fn dispatch(a: Vec<(String, String, String)>, n: usize) -> Vec<String> {
    let b = a.clone();
    let mut selected: Vec<(String, String)> = Vec::new();

    for i in 0..b.len() {
        if selected.len() >= n {
            break;
        }

        if b[i].2 == "ready" {
            let mut found = false;

            for j in 0..selected.len() {
                if selected[j].0 == b[i].0 {
                    found = true;
                }
            }

            if found == false {
                selected.push((b[i].0.clone(), b[i].1.clone()));
            }
        }
    }

    let mut result = Vec::new();

    for i in 0..selected.len() {
        result.push(format!("{}@{}", selected[i].0, selected[i].1));
    }

    result
}

fn main() {
    let input: Vec<(String, String, String)> = [
        ("job-7", "emails", "running"),
        ("job-2", "reports", "ready"),
        ("job-2", "urgent", "ready"),
        ("job-7", "emails", "ready"),
        ("job-9", "exports", "ready"),
    ]
    .into_iter()
    .map(|(id, queue, state)| (id.into(), queue.into(), state.into()))
    .collect();

    let result = dispatch(input, 2);
    println!("{result:?}");
    assert_eq!(result, ["job-2@reports", "job-7@emails"]);
}
```

Run it:

```bash
cargo run
```

Exact program output:

```text
["job-2@reports", "job-7@emails"]
```

## Five refactoring levels

Work through these levels in order, modifying the same implementation. Preserve the behavioral contract at every step. `task_3_final.rs` implements Levels 1–5; hints are available on request.

### Level 1 — Readability · 6 minutes

**Goal:** Improve naming and simplify control flow.

- [x] Replace vague names with names that explain their purpose.
- [x] Remove index-based iteration where the index is unnecessary.
- [x] Make the conditions for skipping a candidate easy to see.
- [x] Preserve selection order, limit handling, and duplicate behavior.

**Done when:** A reader can follow the function without decoding `a`, `b`, `n`, `i`, or `j`. You do not need to change the data types yet.

### Level 2 — Named types · 9 minutes

**Goal:** Make the domain visible in the types.

- [x] Replace the input tuple with a `JobCandidate` struct with named fields.
- [x] Represent the state using `JobState` so selection logic does not compare string literals.
- [x] Keep unknown input states representable and ineligible. You do not need to preserve their original text.
- [x] Preserve exact matching: `"READY"` and `" ready "` must remain ineligible.

**Done when:** Field names explain each value, and eligibility is determined using the enum without changing accepted inputs.

### Level 3 — Selection and presentation · 9 minutes

**Goal:** Separate candidate selection from output formatting.

- [x] Keep eligibility, uniqueness, ordering, and the limit in the selection logic.
- [x] Keep string formatting in a separate function.
- [x] Make selected IDs and queues testable without relying on the `@` separator.
- [x] Keep `dispatch` as the public entry point returning the same formatted rows.

**Done when:** Selection can be checked independently of presentation, and the public output still matches the contract.

### Level 4 — Borrowed input · 12 minutes

**Goal:** Let the caller retain ownership and avoid copying strings during selection.

- [x] Change the API to accept a slice of candidates.
- [x] Use borrowed references to candidates in the selection result.
- [x] Remove copies of the input collection, job IDs, and queue names during reading and selection.
- [x] Keep the final formatted output owned as `Vec<String>`.
- [x] Call `dispatch` twice with the same input without cloning it.

**Done when:** Selection does not copy strings, the caller can reuse the input, and formatting still produces owned strings.

The Level 4 implementation accepts `&[JobCandidate]` and returns `Vec<&JobCandidate>` from selection. These references point to the original candidates, so selection copies neither IDs nor queue names. The selection vector allocates storage for references, and `format_jobs` creates the owned output strings. Selected references cannot outlive the input, while formatted rows can. `main` calls `dispatch` twice with the same borrowed input and prints the original output once. Tests verify candidate identity, unchanged and reusable input, and output surviving the input's scope.

Check this implementation from the workspace root:

```bash
cargo test -p task_3 --bin final
cargo fmt --all --check
cargo clippy -p task_3 --bin final -- -D warnings
cargo run -p task_3 --bin final
```

### Level 5 — Efficient deduplication · 14 minutes

**Goal:** Remove repeated scans for duplicates while preserving deterministic output order.

- [x] Choose an appropriate standard-library data structure to track duplicates.
- [x] Preserve the first eligible occurrence and its position in the output.
- [x] Avoid copying job IDs just to track duplicates.
- [x] Add regression tests for the scenarios below.
- [x] Explain time and auxiliary-space complexity before and after the change.
- [x] Distinguish candidate count, selected-job count, and work on string bytes. State whether your analysis covers the inspected prefix or the entire input, including the original upfront clone.

**Done when:** Each candidate no longer requires a fresh scan of the entire selection. Output order does not depend on hash-collection iteration order, and all regression scenarios satisfy the contract.

The Level 5 solution tracks selected IDs in a `HashSet<&str>`. Only ready candidates reach `insert`; its boolean result rejects duplicate IDs without scanning the selection. The set borrows IDs from the input, while `Vec<&JobCandidate>` records successful selections in input order. Formatting iterates over this vector, so hash-set iteration order never affects the output. The collections grow as jobs are selected, without reserving space for the entire input or the requested limit.

The tests cover every scenario below, plus unknown and empty states, IDs that differ only in surrounding whitespace, and a limit that excludes later unique jobs. A larger test processes 15,000 candidates in three passes: running jobs, their first ready occurrences, and ready duplicates in different queues. It checks all 5,000 selected references and their formatted order without relying on timing assertions. Existing tests also verify borrowing, input reuse, and independently owned output.

### Complexity comparison

The bounds below describe `dispatch`, including formatting but excluding input construction and conversion into `JobState`. Define:

- `n`: all input candidates.
- `p`: candidates examined for eligibility before the limit is reached, at most `n`. For a zero limit, `p = 0`.
- `k`: selected unique jobs, at most `min(p, limit)`.
- `B`: bytes in all original input strings, including IDs, queues, and textual states.
- `L`: maximum ID byte length in the inspected prefix, or zero for an empty prefix.
- `F`: total bytes in the formatted output, including the `@` separators.

| Version | Time upper bound | Peak auxiliary space |
| --- | --- | --- |
| Original upfront clone and linear deduplication | `O(n + B + p*k*(1 + L) + F)` | `O(n + B)` |
| Level 4 borrowed input and linear deduplication | `O(p + p*k*(1 + L) + F)` | `O(k)` |
| Level 5 borrowed input and hash-based deduplication | Expected amortized `O(p*(1 + L) + F)` | `O(k)` |

These are maximum-ID-length bounds. A string comparison or hash is not a constant-time operation for arbitrarily long IDs: its work depends on the bytes it reads. Linear deduplication performs up to `p*k` comparisons, with long shared prefixes making comparisons more expensive. Hashing reads each eligible ID's bytes; table growth can also hash stored IDs again. Expected amortized constant table operations per candidate give the bound above when each operation's string work is bounded by `O(1 + L)`.

For bounded-length IDs, selection improves from `O(p + p*k)` to expected amortized `O(p)`. If every inspected job is ready and unique and the limit permits all of them, `k = p`, so the linear scan performs `p*(p - 1)/2` comparisons. The hash bound is expected, not a worst-case guarantee: severe collisions can cause up to `O(p*k)` equality checks again.

The original version clones the entire collection before inspecting any candidates, so its `O(n + B)` time and temporary storage remain even for a zero limit or a short inspected prefix. It also copies selected IDs and queues, whose bytes are bounded by `B`. Levels 4 and 5 borrow the input and only process the inspected prefix; neither copies input strings. Both the selection vector and the hash set use `O(k)` entries in Level 5, storing references rather than ID bytes.

Auxiliary space excludes the original input and the final owned output. Formatting separately requires `O(k + F)` time and output storage in all versions. Removing string copies saves allocations and byte copying; replacing repeated scans with hashing also changes the expected selection-time growth with candidate and batch size.

## Regression scenarios

Each record below uses `(job_id, queue, state)` notation. Adapt the test fixtures to your refactored types.

| Input records | Limit | Expected output |
| --- | ---: | --- |
| Empty input | 3 | `[]` |
| `("a", "emails", "ready")` | 0 | `[]` |
| `("a", "emails", "running")` | 3 | `[]` |
| `("a", "emails", "READY")` | 3 | `[]` |
| `("a", "emails", " ready ")` | 3 | `[]` |
| `("a", "slow", "ready")`, `("a", "fast", "ready")` | 3 | `["a@slow"]` |
| `("a", "slow", "running")`, `("a", "fast", "ready")` | 3 | `["a@fast"]` |
| `("b", "q", "ready")`, `("a", "q", "ready")` | 3 | `["b@q", "a@q"]` |
| `("a", "q", "ready")`, `("a", "q", "ready")`, `("b", "q", "ready")` | 2 | `["a@q", "b@q"]` |
| `("a", "q", "ready")`, `("A", "q", "ready")` | 3 | `["a@q", "A@q"]` |
| `("", "", "ready")`, `("", "other", "ready")` | 3 | `["@"]` |
| `(" a ", " q ", "ready")` | 3 | `[" a @ q "]` |
| `("a", "q", "ready")`, `("a", "q", "running")` | 3 | `["a@q"]` |

Also verify that the original example still produces the exact expected output, and that two calls with the same borrowed input return the same result while leaving the input unchanged.

Run your checks:

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy -p task_3 --bin final -- -D warnings
cargo run -p task_3 --bin final
```

Run these from the workspace root. The `start` binary retains intentional lint violations and is excluded from the Clippy check.

## Short defense

Why might a linear scan for duplicates be a reasonable choice for a very small batch? What would you need to benchmark to choose between it and a hash-based collection?

For very small batches, a linear scan has no hash-table allocation or hashing overhead and may be faster. Benchmark both versions with representative limits, inspected-prefix lengths, ready and duplicate rates, ID lengths and shared prefixes, and queue lengths. Measure selection separately from complete dispatch so output formatting does not hide the deduplication cost. No benchmark results are claimed here.

## Completion summary

- **Behavior preserved:** Exact ready-state matching, first eligible ID across queues, input order, unique-job limits, opaque IDs and queues, and the original formatted output.
- **Types introduced and why:** `JobCandidate` names the input fields; `JobState` represents readiness and unknown states. Selection returns references to candidates so its fields can be tested directly.
- **Copies removed:** No collection clone or copied IDs and queues during selection; the hash set also borrows its IDs. Only presentation creates owned output strings.
- **How output order is preserved:** Append successful selections to a vector while reading input; never iterate over the hash set to produce output.
- **Regression cases covered:** Every table scenario, additional state and whitespace cases, the original example, borrowing and input reuse, output ownership, and a larger duplicate batch.
- **Complexity before and after, with assumptions:** The comparison above accounts for the original full-input clone, inspected-prefix and selection counts, ID byte lengths, output bytes, and expected amortized hash behavior.
- **Tradeoff I would defend in a review:** A borrowed hash set adds `O(k)` entries and hash-table overhead in exchange for avoiding repeated selection scans. Small batches may favor the simpler linear scan; representative benchmarks should decide that crossover.

Keep the scope focused on this exercise. Introduce abstractions only when they solve a concrete problem in the starting code.
