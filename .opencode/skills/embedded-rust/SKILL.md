---
name: embedded-rust
description: Conventions for writing and reviewing bare-metal no_std Rust firmware for the Raspberry Pi Pico (RP2040). Use when adding or changing firmware code, drivers, peripherals, or HAL usage in this project.
license: MIT
compatibility: opencode
metadata:
  audience: firmware-developers
  stack: rust-no-std-rp2040
---

## What I do

- Apply `no_std` bare-metal Rust patterns targeting `thumbv6m-none-eabi`.
- Guide peripheral access through `rp2040-hal` / `embedded-hal` traits.
- Keep firmware deterministic, panic-free on the hot path, and readable.
- Isolate hardware behind thin wrappers so behavior logic stays testable.

## When to use me

Use me when writing or reviewing firmware code: GPIO, PWM (servos), UART
(audio module), timers, sensors, or the main loop.

## Rules

- Stay `no_std`; never pull in `std` or an OS dependency.
- Prefer `embedded-hal` traits over raw register access.
- No `unwrap()` on fallible hardware calls in the hot path; handle or fall back.
- One peripheral per wrapper module; logic never touches registers directly.
- Keep the main loop a dispatcher for the state machine (see `state-machine`).
- Justify every new crate dependency in the PR/commit.

## Workflow

1. Read `docs/STACK.md` and `docs/PRINCIPLES.md` first.
2. Implement behind a hardware wrapper, then wire it into the FSM.
3. Run `make lint` and `make test` before finishing.
4. Flash with `make upload-firmware`; inspect behavior with `make logs`.
