---
name: firmware-workflow
description: Build, flash, and inspect the Scary Monster Circuit firmware using the project Makefile. Use when uploading firmware to the Pico or opening device logs during development.
license: MIT
compatibility: opencode
metadata:
  audience: firmware-developers
  tooling: make
---

## What I do

- Build and flash the firmware onto the Raspberry Pi Pico.
- Open the device logs over RTT.
- Keep the build/flash cycle one command, per the buildability principle.

## When to use me

Use me when you need to run firmware on hardware or read its runtime output.

## Commands

- `make upload-firmware` — build and flash the firmware onto the device.
- `make logs` — open the device logs over RTT.
- `make help` — list all available targets.

## Rules

- Always run `make lint` and `make test` before `make upload-firmware`.
- Use `make logs` to confirm runtime behavior; do not guess.
- The toolchain is `cargo` + `probe-rs` targeting `thumbv6m-none-eabi`.
- If a command is missing or wrong, update the Makefile rather than running
  ad-hoc commands by hand.

## Troubleshooting

- Device not found: check USB connection and that `probe-rs` is installed.
- Flash fails: confirm the chip (`RP2040`) and target match `docs/STACK.md`.
- No logs: ensure the firmware is running and RTT/`defmt` is enabled.
