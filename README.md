# Scary Monster Circuit

Bare-metal Rust firmware for the Halloween prop.

## Prerequisites

- Rust toolchain (`rustup`)
- Pico target: `rustup target add thumbv6m-none-eabi`
- `probe-rs` for flashing and logs: `cargo install probe-rs-tools`
- Coverage tools (optional, for `make coverage`): `rustup component add llvm-tools-preview`
- Python 3 (only for `make schematics`; the tool creates its own `.venv`)

## Quick start

```sh
make upload-firmware
```

## Makefile targets

```sh
make help            # list all targets
make upload-firmware # build and flash the firmware onto the Pico
make logs            # open the device logs over RTT
make test            # run the unit tests on the host
make coverage        # report unit-test code coverage
make lint            # run the linter (firmware and tests)
make lint-fix        # auto-fix lint problems
make schematics      # rebuild the schematics PDF
```
