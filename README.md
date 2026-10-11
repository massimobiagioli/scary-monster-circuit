# Scary Monster Circuit

Bare-metal Rust firmware for the Halloween prop.

## Prerequisites

- Rust toolchain (`rustup`)
- Pico target: `rustup target add thumbv6m-none-eabi`
- `elf2uf2-rs` for USB flashing: provided by `tools/elf2uf2-rs`, which installs
  the pinned version (see `tools/elf2uf2-rs/version.txt`) into a local root on
  first use. No global `cargo install` is needed.
- Logs need no extra tool: the firmware streams them as a USB serial port, so
  `make logs` works with nothing but the USB cable.
- `probe-rs` (optional): provided by `tools/probe-rs` for flashing over SWD with
  a real debug probe. Not needed for the normal USB workflow.
- Coverage tools (optional, for `make coverage`): `rustup component add llvm-tools-preview`
- Python 3 (only for `make schematics`; the tool creates its own `.venv`)

## Quick start

Put the Pico in BOOTSEL mode: hold the **BOOTSEL** button while plugging the USB
cable. Then flash it and open the logs:

```sh
make upload-firmware
make logs
```

## Makefile targets

```sh
make help            # list all targets
make build           # build the release firmware
make upload-firmware # build and flash the firmware over USB (BOOTSEL)
make logs            # stream the live dashboard over USB serial
make probe           # build, flash, and stream the dashboard in one command
make test            # run the unit tests on the host
make coverage        # report unit-test code coverage
make lint            # run the linter (firmware and tests)
make lint-fix        # auto-fix lint problems
make schematics      # rebuild the schematics PDF
```
