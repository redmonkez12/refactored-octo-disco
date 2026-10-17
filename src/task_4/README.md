# Rust Refactoring Exercise 04: Bounded FIFO Cache

**Estimated time:** 50 minutes  
**Dependencies:** Rust standard library only

Practice reading, refactoring, and explaining existing Rust code through five successive levels. This is a standalone exercise, separate from `stack-lab`.

This original exercise is inspired by a progressive interview format. It is not an actual or confidential Better Stack interview assignment.

## Backend context

A backend keeps a small cache of results from expensive computations. When the cache is full, inserting a new key evicts the entry that has been stored the longest.

The starting implementation works, but it repeats searches, copies strings unnecessarily, and shifts the remaining entries when removing the first one.

## Behavioral contract

Preserve the following behavior unless a level explicitly changes the API:

- The cache starts empty and uses the same capacity throughout its lifetime.
- Keys are unique, case-sensitive, opaque strings. Do not trim or normalize them.
- Values may be empty strings.
- Inserting a new key appends an entry to the end.
- If the cache is full, inserting a new key first removes the oldest entry.
- Updating an existing key changes its value only. **It does not move the entry to the end.**
- Reading an entry does not change its position. Eviction follows FIFO (first in, first out), regardless of reads or updates.
- Writing an empty key returns the error `"empty_key"`.
- Writing to a zero-capacity cache returns the error `"disabled_cache"`.
- If both errors apply, `"empty_key"` takes priority.
- A failed write must leave both the contents and the ordering unchanged.
- Reading a missing key returns `None`. Reading an empty key also returns `None`.
- The initial `get` returns an owned copy of the value. **Level 4 explicitly changes this API to return a borrowed value.**

For the starting implementation, create an empty vector and access it through the provided functions. Keep the capacity argument constant for that cache. Level 3 will make these invariants the responsibility of the cache type.

## Local setup

Create a standalone Cargo project:

```bash
cargo new bounded-fifo-cache
cd bounded-fifo-cache
```

Replace `src/main.rs` with the starting code below. No additional dependencies, accounts, or infrastructure are required.

## Starting code

```rust
fn put(
    c: &mut Vec<(String, String)>,
    n: usize,
    k: String,
    v: String,
) -> Result<(), String> {
    if k.is_empty() {
        return Err("empty_key".to_string());
    }
    if n == 0 {
        return Err("disabled_cache".to_string());
    }

    let mut found = false;
    for i in 0..c.len() {
        if c[i].0 == k {
            found = true;
        }
    }

    if found == true {
        for i in 0..c.len() {
            if c[i].0 == k {
                c[i].1 = v.clone();
            }
        }
    } else {
        if c.len() >= n {
            c.remove(0);
        }
        c.push((k.clone(), v.clone()));
    }

    Ok(())
}

fn get(c: &Vec<(String, String)>, k: String) -> Option<String> {
    for i in 0..c.len() {
        if c[i].0 == k {
            return Some(c[i].1.clone());
        }
    }

    None
}

fn main() {
    let mut cache = Vec::new();
    let capacity = 2;

    put(&mut cache, capacity, "a".into(), "one".into()).unwrap();
    put(&mut cache, capacity, "b".into(), "two".into()).unwrap();
    put(&mut cache, capacity, "a".into(), "updated".into()).unwrap();
    put(&mut cache, capacity, "c".into(), "three".into()).unwrap();

    println!("a={:?}", get(&cache, "a".into()));
    println!("b={:?}", get(&cache, "b".into()));
    println!("c={:?}", get(&cache, "c".into()));
}
```

Run the example:

```bash
cargo run
```

Exact expected program output:

```text
a=None
b=Some("two")
c=Some("three")
```

Updating `a` leaves it in the oldest position, so inserting `c` evicts it.

## Five refactoring levels

Complete the levels in order, modifying the same implementation. `task_4_start.rs` contains the starting version, and `task_4_final.rs` contains the completed Levels 1–5.

### Level 1 — Readability and duplicate work · 7 minutes

**Goal:** Make the functions easier to read and avoid searching twice for the same key.

- [ ] Rename parameters to explain their purpose.
- [ ] Remove unnecessary index-based iteration and comparisons with `true`.
- [ ] Avoid a second search when updating an existing entry.
- [ ] Preserve validation order and leave the cache unchanged on errors.

**Done when:** The update and insert paths are easy to distinguish, and finding an existing entry does not require another search.

### Level 2 — Named entries and errors · 9 minutes

**Goal:** Replace the tuple with `CacheEntry` and string errors with `CacheError`.

- [ ] Give each entry named fields.
- [ ] Represent empty-key and disabled-cache errors as distinct enum variants.
- [ ] Implement `Display` to preserve the error codes `empty_key` and `disabled_cache`.
- [ ] Keep the original error priority.

**Done when:** Domain logic no longer uses `.0` and `.1`. Callers can distinguish error variants, and their text representation remains unchanged. Make the error type usable with the example's `unwrap` calls.

Changing the error type is an intentional API change. Validation rules remain unchanged.

### Level 3 — Cache invariants · 10 minutes

**Goal:** Introduce a `Cache` type that owns both its entries and its capacity. Move the operations into methods.

- [ ] Ensure every new cache starts empty.
- [ ] Set capacity at construction and keep it fixed through the public API.
- [ ] Keep entries private so callers cannot directly insert duplicates or exceed capacity.
- [ ] Allow construction with zero capacity; writes must still return the appropriate error.
- [ ] Remove the capacity argument from individual writes.

**Done when:** Callers cannot break the invariants through the public API and do not pass capacity to every write. Observable cache behavior and error priority remain unchanged.

### Level 4 — Ownership at the boundary · 12 minutes

**Goal:** Remove string copies that the cache does not need.

- [ ] Keep owned keys and values as write inputs; move them into storage where they need to be retained.
- [ ] Accept a borrowed key for reads.
- [ ] **New requirement:** Return `Option<&str>` from reads, borrowing the stored value.
- [ ] Preserve the example's printed output.
- [ ] Explain why a reference returned by a read cannot outlive the cache.

**Done when:** Cache reads and writes use no `clone`. Reading does not allocate a copy of the value, and the returned reference is tied to the cache's lifetime.

This level deliberately changes the ownership contract of the read API. The returned text and all FIFO rules remain unchanged.

**Short explanation:** Why can't you keep a reference returned by `get`, mutate the cache, and then use that reference again?

### Level 5 — FIFO storage and regression tests · 12 minutes

**Goal:** Choose a standard-library collection that can remove the oldest entry without shifting all remaining entries.

- [x] Preserve entry positions during reads and updates.
- [x] Add regression tests covering the scenarios below.
- [x] Explain the complexity of lookup, update, and eviction before and after this change.
- [x] Distinguish the cost of eviction alone from the cost of an entire write.
- [x] State your assumptions about entry count and the cost of comparing or destroying strings.

**Done when:** Evicting the oldest entry no longer shifts every remaining entry. You can explain which operations are still linear and why.

Constant-time key lookup is not required.

## Regression scenarios

Start each scenario with a fresh cache. `put(a, one)` means writing the key `"a"` with the value `"one"`. Adapt fixtures to your final API.

| Capacity | Operations | Expected behavior |
| ---: | --- | --- |
| 2 | Read from an empty cache | `None` |
| 2 | `put(a, one)`, `get(a)` | `Some("one")` |
| 2 | `put(a, one)`, `put(a, two)` | One entry; its value is `"two"` |
| 2 | `put(a, one)`, `put(b, two)`, `put(c, three)` | `a` is missing; `b` and `c` remain |
| 2 | `put(a, one)`, `put(b, two)`, `put(a, updated)`, `put(c, three)` | `a` is evicted |
| 2 | `put(a, one)`, `put(b, two)`, `get(a)`, `put(c, three)` | `a` is evicted |
| 1 | `put(a, one)`, `put(b, two)` | Only `b` remains |
| 2 | `put(a, "")`, `get(a)` | `Some("")` |
| 2 | Write an empty key to a full cache | `empty_key`; contents and order unchanged |
| 0 | Write a nonempty key | `disabled_cache`; cache remains empty |
| 0 | Write an empty key | `empty_key` |
| 2 | Write keys `"a"` and `"A"` | Two distinct entries |
| 2 | Write keys `"a"` and `" a "` | Two distinct entries |

For a failed write, verify ordering through a later valid insertion as well: the originally oldest entry must still be evicted.

Run the final implementation's checks from the repository root:

```bash
rustfmt --check --edition 2024 src/task_4/task_4_final.rs
cargo test -p task_4 --bin final
cargo run -p task_4 --bin final
cargo clippy -p task_4 --bin final -- -D warnings
```

## Complexity and assumptions

Let `n` be the number of stored entries, bounded by the configured capacity. The comparison below is between Level 4's `Vec<CacheEntry>` and Level 5's `VecDeque<CacheEntry>`, with the same borrowed read API and no string cloning.

| Operation | Before: `Vec` | After: `VecDeque` |
| --- | --- | --- |
| Find a key or read its value, worst case | O(n) | O(n) |
| Update an existing key, including lookup, worst case | O(n) | O(n) |
| Remove the oldest entry alone | O(n), shifts remaining entries | O(1), moves the front position |
| Append an entry, excluding lookup and eviction | O(1) amortized | O(1) amortized |
| Entire write of a new key, including lookup and possible eviction | O(n) | O(n) |

The table treats key comparison and string destruction/deallocation as O(1), and excludes constructing the owned input strings at the caller. If a comparison examines up to `L` bytes, worst-case lookup becomes O(nL); updates and new-key writes include that lookup cost. Add the cost of releasing discarded strings to updates, evictions, and writes. Moving a `String` transfers its descriptor without copying its text.

Appending can occasionally grow storage and move O(n) entry descriptors, hence the amortized bound. When the cache is already full, removing the front entry before appending reuses available storage. Eviction itself no longer moves the other entries, but a complete write still searches for the key first. This improves the eviction work without making writes or reads constant time.

## Short defense

If you make oldest-entry eviction faster but key lookup remains linear, what have you actually improved? When would an additional lookup index justify the extra implementation complexity?

`VecDeque` removes the linear shift during eviction while keeping FIFO order explicit. A lookup index becomes worthwhile when measurements show that scans dominate for larger caches or frequent reads and updates. A hash index could give expected constant-time lookup with bounded key costs, but adds memory, hashing, and the need to keep the index consistent with FIFO storage during updates and eviction.

## Completion summary

- **Behavior preserved:** Unique, case-sensitive, unmodified keys; empty values; FIFO eviction; fixed capacity; and the original example output and error codes.
- **Intentional API changes:** `Cache::new(capacity)` owns private entries and capacity; `put` returns `Result<(), CacheError>`; `get(&str)` returns `Option<&str>` borrowed from the cache.
- **Duplicate work removed:** A single mutable search finds and updates an existing entry directly.
- **Ownership decisions:** Writes move owned keys and values into storage. Reads borrow both the lookup key and the stored value, without cloning or allocating. The returned reference cannot outlive the cache, and a write cannot borrow the cache mutably while that reference will still be used.
- **How FIFO order is preserved:** New entries use `push_back`; eviction uses `pop_front`. Reads and updates leave entry positions unchanged.
- **How failed writes leave state unchanged:** Empty-key validation runs before zero-capacity validation, and both run before any mutation or eviction.
- **Regression cases covered:** All 13 listed scenarios, plus independent lookup-key lifetime, missing-key reads in a populated cache, and repeated evictions and updates.
- **Complexity before and after, with assumptions:** See the table above. Eviction changes from O(n) to O(1) under constant string-destruction costs; lookup, update, and entire writes remain linear in entry count under constant key-comparison costs.
- **Tradeoff I would defend in a review:** `VecDeque` avoids shifting entries on eviction with a small implementation change. Keep linear lookup for a small cache until measurements justify an additional index.

Keep the scope focused on this cache. Add abstractions only when they solve a concrete problem in the starting code.
