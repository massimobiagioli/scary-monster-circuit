---
name: rust-quality
description: Run unit tests, the linter, and automatic lint fixes for the Scary Monster Circuit firmware using the project Makefile. Use before finishing any code change to keep the build clean and readable.
license: MIT
compatibility: opencode
metadata:
  audience: firmware-developers
  tooling: make
---

## What I do

- Run the unit tests.
- Measure the code coverage of the library.
- Run the linter with warnings treated as errors.
- Auto-fix lint problems and format the code.

## When to use me

Use me after any code change, and before flashing or opening a PR.

## Commands

- `make test` — run the unit tests on the host.
- `make coverage` — report line/region coverage with `llvm-cov`.
- `make lint` — run the linter on the firmware and on the tests (`cargo clippy -D warnings`).
- `make lint-fix` — auto-fix lint problems and run `cargo fmt`.
- `make help` — list all available targets.

## Rules

- A change is not done until `make test` and `make lint` both pass.
- Prefer `make lint-fix` for mechanical issues, then re-run `make lint`.
- Never silence a warning without a documented reason.
- Keep code formatted with `cargo fmt` (included in `make lint-fix`).
- Every new module of the library ships with unit tests.
- `cargo test --lib` only builds the library, so **all testable logic belongs
  in `firmware/src/lib/`** (see `controller.rs`), never in `src/main.rs`.
- Use the mocks in `firmware/src/lib/test_support.rs`; do not talk to real
  hardware or the OS in tests.

## Coverage

`make coverage` needs the `llvm-tools-preview` rustup component:

```sh
rustup component add llvm-tools-preview
```

The goal is 100% of regions, functions and lines for every module under
`firmware/src/lib/`. Test code, the crate root and third-party crates are
excluded from the report.

## Order of operations

1. `make lint-fix`
2. `make lint`
3. `make test`
4. `make coverage` (when you touch library code)
