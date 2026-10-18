# Rust Refactoring Exercise 05: HTTP Retry Planner

**Estimated time:** 50 minutes  
**Dependencies:** Rust standard library only

Practice reading, refactoring, and explaining existing Rust code through five successive levels. This is an original practice exercise, not an actual or confidential Better Stack interview assignment.

## Backend context

A backend processes HTTP request results and decides whether to accept, retry, reject, give up on, or ignore each result. Each input tuple contains an endpoint, an HTTP status code, and the number of attempts already made.

The starting implementation clones the entire input and copies endpoint strings before inspecting them. Its short parameter names and index-based iteration make the validation and decision rules harder to follow.

## Files and local setup

This task is already part of the workspace:

- `task_5_start.rs` contains the original implementation.
- `task_5_final.rs` contains the completed Levels 1–5 and regression tests for decision boundaries, ordering, repeated and opaque endpoints, borrowed input, owned output, validation, and larger inputs.

Run commands from the workspace root:

```bash
cargo run -p task_5 --bin start
cargo run -p task_5 --bin final
cargo test -p task_5 --bin final
cargo clippy -p task_5 --bin final -- -D warnings
rustfmt --edition 2024 --check src/task_5/task_5_final.rs
```

## Behavioral contract

Preserve these rules throughout the refactor:

- The initial API accepts `Vec<(String, u16, u32)>` and a `u32` maximum attempt count, returning `Result<Vec<String>, String>`.
- Level 2 intentionally changes the input to `Vec<RequestResult>` and the return type to `Result<Vec<String>, PlanError>`. Error text remains unchanged through `Display`.
- Level 4 intentionally changes the input to `&[RequestResult]`. The caller retains ownership, while the output remains an independently owned `Vec<String>`.
- A zero maximum attempt count returns `invalid_limit`, even for empty input or invalid records.
- Process records in input order and stop at the first invalid record.
- Valid HTTP status codes are `100..=599`. Reject other codes with `invalid_status:{index}`.
- Attempts must be greater than zero. Reject zero with `invalid_attempts:{index}`.
- Error indices are zero-based positions in the original input, not status codes or attempt counts.
- Validate the limit first; within each record, validate status before attempts.
- On error, return only the error, without exposing a partial plan.
- A valid limit with empty input returns an empty vector.
- Preserve endpoint text exactly, including empty strings, whitespace, case, and colons. Do not deduplicate endpoints.
- Return one `endpoint:action` string per valid record, preserving input order.
- Attempts may equal or exceed the maximum; this is not a validation error.

Apply decision rules in this order:

| Status | Action |
| --- | --- |
| `200..=299` | `accept` |
| `429` or `500..=599`, attempts below the maximum | `retry` |
| `429` or `500..=599`, attempts at or above the maximum | `give_up` |
| Other `400..=499` | `reject` |
| `100..=199` or `300..=399` | `ignore` |

The special handling of `429` must take priority over the general client-error rule.

Both example binaries must print:

```text
/billing:retry
/export:give_up
/users:accept
/auth:reject
/redirect:ignore
```

## Five refactoring levels

Complete the levels in order, modifying `task_5_final.rs`. Keep `task_5_start.rs` as the original reference.

### Level 1 — Readability and duplicate work · 7 minutes

**Goal:** Make the processing flow clear and remove unnecessary input copies.

- [x] Rename parameters to explain their purpose.
- [x] Replace index-based tuple access with direct destructuring.
- [x] Avoid cloning the input collection and endpoint strings just to process owned input.
- [x] Keep the original record index available for error messages using `enumerate`.
- [x] Preserve validation order, error text, decision rules, and output order.

**Done when:** A reader can identify validation and action selection without decoding short variable names. Error messages still identify the original input position.

The Level 1 solution uses `request_results` and `max_attempts`, consumes input records through `into_iter().enumerate()`, and retains string-based actions and errors for later levels. Tests cover both indexed errors at several positions, error priority, empty input, and the example output.

### Level 2 — Named results and errors · 9 minutes

**Goal:** Express the input and validation failures with named types.

- [x] Replace the input tuple with a `RequestResult` struct containing named fields.
- [x] Introduce a `PlanError` enum for an invalid limit, status, and attempt count.
- [x] Retain the original input index in record-specific error variants.
- [x] Implement `Display` to preserve the exact error text, and support the example's `unwrap` calls.
- [x] Preserve validation priority and action selection.

**Done when:** Domain logic no longer depends on tuple positions, and callers can distinguish validation errors without parsing strings. Changing the input and error types is intentional at this level.

The Level 2 implementation reads the named `endpoint`, `status`, and `attempts` fields. `PlanError` distinguishes `ZeroLimit`, `InvalidStatus { index }`, and `ZeroAttempts { index }`. Its `Display` implementation preserves the original error codes; derived `Debug` supports `unwrap`, while `PartialEq` and `Eq` allow tests to assert variants and indices directly. The example and tests use `RequestResult` inputs, and the example's output is unchanged.

### Level 3 — Decisions and presentation · 10 minutes

**Goal:** Separate action selection from output formatting.

- [x] Represent the five actions with an `Action` enum.
- [x] Isolate the decision rules so they can be tested without formatted endpoint strings.
- [x] Keep validation order explicit, including the special handling of `429`.
- [x] Keep `plan` as the entry point returning the same formatted rows on success.

**Done when:** Tests can assert actions directly, while formatted output and error behavior remain unchanged.

The Level 3 implementation uses `select_action` to return an `Action` without formatting an endpoint. `plan` still validates the limit first, then each record's status and attempts before selecting its action. The `429` match arms precede the general client-error arm. `format_decision` creates each output row using `Display` for `Action`, with no intermediate action strings. Tests assert enum variants at status and retry-limit boundaries and verify endpoint text independently of the decision rules. The input ownership and `plan` signature remain unchanged from Level 2.

### Level 4 — Borrowed input · 12 minutes

**Goal:** Allow callers to reuse input without copying endpoint strings.

- [x] Accept a borrowed slice of request results.
- [x] Borrow endpoint strings during validation and decision making.
- [x] Retain owned `Vec<String>` output so formatted rows can outlive the input.
- [x] Call the planner twice with the same input without cloning it.
- [x] Explain which values borrow from the input and which allocations formatting requires.

**Done when:** The caller retains ownership, input processing does not clone strings, and returned formatted rows do not depend on the input's lifetime. Borrowing the input is an intentional API change at this level.

The Level 4 implementation accepts `&[RequestResult]` and iterates with `iter()`, so each inspected record is a shared reference to the caller's data. Validation and action selection do not clone endpoint strings; numeric fields are copied, and `format_decision` temporarily borrows the endpoint as `&str`. The caller can pass a vector or an array as a slice and reuse it after the call.

The output vector allocates storage for its rows, and formatting allocates each owned `String`, copying the endpoint bytes and action text into the final output. These output copies let the rows outlive the input; there are no temporary endpoint clones. The returned value contains no references to the input, so the caller can mutate or drop the input after `plan` returns while retaining the output. `main` calls the planner twice on the same input without cloning and prints the output once. Tests verify reuse with different limits, unchanged input fields, and output surviving both input mutation and the input's scope.

### Level 5 — Regression tests and complexity · 12 minutes

**Goal:** Defend the behavior and cost of the final implementation.

- [x] Cover the boundary and regression scenarios below.
- [x] Verify error indices after valid input prefixes and preserve error priority.
- [x] Verify output order and repeated endpoints.
- [x] Explain time and allocation costs before and after the refactor.
- [x] Distinguish record count, the inspected prefix before an error, endpoint bytes, and output bytes.

**Done when:** Tests protect decision boundaries and validation behavior, and the complexity explanation includes the original upfront clone and the allocations still needed for output.

The completed suite covers every scenario below. Additional cases exercise the extremes of `u16` and `u32`, multi-digit error indices, Unicode endpoint text, and reuse after repairing invalid input. A larger regression checks all 10,000 output rows, including repeated endpoints and every action, then appends an invalid record and verifies the error at index 10,000. The tests make no timing assertions; the complexity analysis follows the operations in the implementation.

## Regression scenarios

Use a fresh input for each scenario and preserve the maximum attempt count specified by the test.

| Scenario | Expected behavior |
| --- | --- |
| Empty input, maximum `3` | Empty output |
| Empty input or invalid records, maximum `0` | `invalid_limit` |
| Status `99` or `600` at position `i` | `invalid_status:i` |
| Status `503`, zero attempts at position `i` | `invalid_attempts:i` |
| Invalid status and zero attempts in one record | Status error takes priority |
| Zero attempts in an earlier record, invalid status in a later record | Earlier attempts error takes priority |
| Status boundaries `100`, `199`, `200`, `299`, `300`, `399`, `400`, `499`, `500`, `599` | Actions follow the decision table |
| Status `429`, attempts `2`, maximum `3` | `retry` |
| Status `429`, attempts `3` or `4`, maximum `3` | `give_up` |
| Status `503`, attempts `2`, maximum `3` | `retry` |
| Status `503`, attempts `3` or `4`, maximum `3` | `give_up` |
| Success status with attempts above the maximum | `accept` |
| Repeated endpoints with different results | Separate rows in original order |
| Empty endpoint or endpoint containing spaces or colons | Exact endpoint text retained |

## Complexity and allocations

Analyze one `plan` call, excluding input construction and console printing, and including cleanup performed inside that call. Define:

- `n`: all input records.
- `p`: records inspected, including the first invalid record on failure. On success, `p = n`; with a zero limit, `p = 0`.
- `k`: rows formatted before returning. On success, `k = p = n`; on a record validation error, `k = p - 1`.
- `B`: total endpoint bytes in the entire input, measured as UTF-8 bytes rather than characters.
- `B_p`: endpoint bytes in the inspected prefix, including the invalid record when present.
- `R`: total bytes in the `k` formatted rows, including endpoints, separators, and action text. On failure these rows are temporary and get discarded.

Assume fixed-size numeric operations and record moves take constant time, action labels have bounded length, and copying or formatting text costs time proportional to its bytes. Assume vector and string growth has amortized linear cost. Count each buffer allocation or deallocation as constant overhead in addition to copied bytes; allocator-specific costs can change practical timings. Dropping a `String` frees its buffer without visiting each byte. Vector and string capacities are assumed to stay within a constant factor of their logical sizes.

For a **nonzero limit**, the bounds are:

| Cost | Starting implementation | Final implementation |
| --- | --- | --- |
| Time for one call | `O(n + B + p + B_p + R)` | `O(p + R)` |
| Peak storage including output or discarded partial output, excluding the supplied input | `O(n + B + k + R)` | `O(k + R)` |
| Auxiliary storage excluding input and output rows | `O(n + B)` | `O(1)` |

Bounds include constant call overhead; empty cases take constant time unless owned input must be destroyed as described below. The original time expression simplifies to `O(n + B + R)` because `p <= n` and `B_p <= B`, but the expanded expression shows the separate copies:

1. `a.clone()` copies all `n` records and their `B` endpoint bytes before any record validation.
2. Each inspected record clones its endpoint again, costing `O(p + B_p)` in total. This includes the endpoint of an invalid record, since that clone precedes validation.
3. Valid records allocate temporary action strings and format the final rows. Formatting costs `O(k + R)`; moving string handles as the output vector grows costs amortized `O(k)`, without copying the strings' contents again.
4. The owned argument and its cloned vector are dropped inside the original function, including on errors. Under the stated deallocation assumption this adds `O(n)` cleanup, already included above.

The final version performs `O(p)` validation and action selection, plus `O(R)` formatting. It allocates the output vector and the buffers for formatted rows, without allocating an input clone, temporary endpoint copies, or action strings. Vector or string growth may require further allocation calls; the claim concerns asymptotic costs, not an exact number of allocator calls. A late validation error still drops the already formatted prefix, adding `O(k)` cleanup under the stated assumptions, which is covered by `O(p + R)`.

On successful nonempty input, `R` includes all endpoint bytes and a bounded amount of text per record, so both versions have `O(n + B)` time complexity. The refactor removes redundant copying and auxiliary storage; it does not make full-output processing sublinear. On an early record error, the final version depends on the inspected prefix and its formatted rows instead of cloning all remaining endpoints. A very long invalid endpoint is not copied or scanned by the final validator, since validation reads only numeric fields.

With a **zero limit**, neither version clones or inspects records. The final version returns a typed error in `O(1)` time and space. The original creates a constant-length error string but still destroys its owned input, taking `O(n)` time under the cleanup assumption, with `O(1)` additional storage. If input destruction were excluded, the original validation branch alone would also be constant time. The caller's eventual destruction of borrowed input is outside the final function's cost.

`PlanError` stores only a variant and possibly an index, so constructing it allocates no string. Its text is formatted only when the caller requests `Display`; indexed error formatting is proportional to the number of decimal digits written, and `to_string()` allocates the resulting text.

## Review tradeoff

The implementation validates and formats in one pass. A later error means earlier formatted rows were allocated and then discarded, even though no partial output escapes. A separate validation pass could avoid those output allocations on errors, at the cost of another traversal on successful input. The current design keeps ordering and error priority explicit, and performs no extra work on uninspected endpoint text.

## Completion summary

All five levels are complete:

- **Highest level completed:** Level 5.
- **Behavior preserved:** Validation priority, zero-based error indices, decisions, and output order.
- **Duplicate work removed:** The input vector and endpoint strings are no longer cloned for processing; action selection no longer allocates temporary action strings.
- **Intentional API changes:** Level 2 replaces input tuples with `RequestResult` and string errors with `PlanError`; error text remains unchanged through `Display`. Level 4 accepts `&[RequestResult]` while retaining owned output.
- **Regression cases covered:** Every listed scenario, numeric extremes, input reuse and preservation, output independence from input mutation and lifetime, opaque and repeated endpoints, late errors and multi-digit indices, and a 10,000-row regression.
- **Ownership decisions:** Records and endpoints are borrowed during processing; numeric fields are copied. The output vector and formatted strings are owned and can outlive the input.
- **Complexity before and after, with assumptions:** For a nonzero limit, original time is `O(n + B + p + B_p + R)` with `O(n + B)` auxiliary storage; final time is `O(p + R)` with `O(1)` auxiliary storage. Output or partial-output storage is `O(k + R)`. See the analysis above for allocation, cleanup, and zero-limit assumptions.
- **Tradeoff I would defend in a review:** One pass preserves simple control flow and prefix processing, while accepting that late errors discard previously formatted rows; a preliminary validation pass could reduce wasted allocations on errors.

Keep the scope focused on HTTP result planning. Add abstractions only when they solve a concrete problem in the starting code.
