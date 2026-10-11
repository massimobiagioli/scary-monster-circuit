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
- Stream the live dashboard of every component over USB serial with `make logs`.
- Keep the build/flash cycle one command, per the buildability principle.

## When to use me

Use me when you need to run firmware on hardware or read its runtime output.

## Commands

- `make build` — build the release firmware.
- `make upload-firmware` — build and flash the firmware over USB (BOOTSEL).
- `make logs` — stream the live dashboard over USB serial.
- `make probe` — build, flash, and stream the dashboard in one command.
- `make help` — list all available targets.

## Rules

- Always run `make lint` and `make test` before `make upload-firmware`.
- Build through `make build` (or `make upload-firmware`, which calls it); never
  call `cargo build` by hand.
- Use `make logs` to confirm runtime behavior; do not guess. The firmware always
  streams a live dashboard with every sensor and actuator value.
- Target is `thumbv6m-none-eabi` (RP2040).
- Flashing over USB uses `elf2uf2-rs` from `tools/elf2uf2-rs/elf2uf2-rs`, so the
  Pico must be in BOOTSEL mode (hold the BOOTSEL button while plugging USB).
- Logs are a USB serial port (CDC); `make logs` finds and reads it. No debug
  probe is needed. `tools/probe-rs` is optional and only for SWD flashing.
- Tools are project-local and pinned (`version.txt`); never run a bare global
  `cargo install`.
- If a command is missing or wrong, update the Makefile rather than running
  ad-hoc commands by hand.

## Troubleshooting

- Flashing can't find a mounted Pico: put the Pico in BOOTSEL mode (hold the
  BOOTSEL button while plugging USB); the tools install themselves on first use.
- `make logs` finds no port: flash first and wait a second for the USB serial
  port to appear (`/dev/cu.usbmodem*` on macOS, `/dev/ttyACM*` on Linux).
- `make logs` stops cleanly on Ctrl-C (`(stopped)`, exit 0); it uses
  `tools/logs/`, which keeps `make` from printing an "Interrupt" error.
- Flash fails: confirm the chip (`RP2040`) and target match `docs/STACK.md`.
- No logs: ensure the firmware that streams over USB is flashed and running.
