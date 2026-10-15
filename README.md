# Refaktoring v Rustu

Každý task má vlastní složku v `src/`, původní verzi `start` a výslednou verzi `final`.

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

## Spuštění

Příkazy spouštěj z kořenové složky projektu.

```sh
# Původní verze (výchozí)
cargo run -p task_1

# Původní verze explicitně
cargo run -p task_1 --bin start

# Výsledná verze
cargo run -p task_1 --bin final

# Kontrola všech tasků
cargo check --workspace
```

Soubor `task_1_final.rs` obsahuje řešení úrovní 1–5: pojmenovaný datový model, oddělené počítání chyb a formátování reportu, vypůjčený vstup bez klonování názvů služeb a agregaci jedním průchodem pomocí `HashMap` s následným seřazením výsledků. Regresní testy pokrývají smluvené chování i větší vstup. Obě verze mají vlastní `fn main()`.

```sh
# Testy a formátování
cargo test --workspace
cargo fmt --all --check

# Clippy pro výslednou verzi (původní start má záměrné prohřešky)
cargo clippy -p task_1 --bin final -- -D warnings
```

## Přidání dalšího tasku

1. Vytvoř složku `src/task_2`.
2. Přidej `task_2_start.rs` a `task_2_final.rs`, každý s vlastní `fn main()`.
3. Přidej do této složky `Cargo.toml`:

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

Pak spusť `cargo run -p task_2` nebo `cargo run -p task_2 --bin final`.
Kořenový `Cargo.toml` není potřeba upravovat: workspace automaticky zahrnuje složky `src/task_*` s vlastním `Cargo.toml`.
