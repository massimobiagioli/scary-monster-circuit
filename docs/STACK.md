# Stack

Technology choices for the Scary Monster Circuit.

## Hardware

- **Board:** Raspberry Pi Pico (RP2040).
- **Language runtime:** Bare-metal Rust (`no_std`), compiled directly to the
  target. No operating system, no interpreter, no vendor SDK in the hot path.
- **Motion sensing:** two side-mounted distance sensors (ultrasonic), wired to
  GPIO pins.
- **Head actuation:** servo motor(s) driven by PWM for pan (and optionally tilt).
- **Audio:** stored scary song played back through a DFPlayer-style module,
  triggered over UART.

## Software

- **Language:** Rust, edition 2021.
- **Target:** `thumbv6m-none-eabi` (RP2040 Cortex-M0+).
- **Crates:**
  - `rp-pico` / `rp2040-hal` — board and peripheral access.
  - `embedded-hal` — portable hardware traits.
  - `embedded-io` — portable I/O traits (UART writes to the MP3 player).
  - `cortex-m` + `cortex-m-rt` — core and runtime.
  - `panic-halt` — panic handler for `no_std`.
  - `usb-device` + `usbd-serial` — USB CDC serial port, so logs reach the host
    over the same USB cable used for flashing.
- **Build/Flash:** `cargo` + `elf2uf2-rs` over USB in BOOTSEL mode (both
  provided project-local by `tools/`). `probe-rs` is optional, only for
  flashing over SWD with a real debug probe.
- **Logs:** the firmware streams plain-text logs over USB serial; `make logs`
  reads that port. No debug probe and no `defmt`/RTT are involved.
- **Config:** `memory.x` linker script and `.cargo/config.toml` for the target.

## Tooling (Python)

- `tools/md2pdf` converts Markdown with Mermaid diagrams into a PDF.
- `tools/logs` streams the firmware's USB serial dashboard (`make logs`) and
  stops cleanly on Ctrl-C without a `make` "Interrupt" error.
- **Every Python tool MUST run inside its own `.venv`.** Never install Python
  packages into the system interpreter.
- Dependencies are pinned in a `requirements.txt` next to each tool, and the
  `.venv` is created automatically on first use (and is git-ignored).

## Development Environment

- `rustup target add thumbv6m-none-eabi`
- Tools are installed project-local on first use, pinned in `version.txt`; never
  run a bare global `cargo install`:
  - `tools/elf2uf2-rs` — flashes the Pico over USB (BOOTSEL mode), used by
    `make upload-firmware`.
  - `tools/probe-rs` — optional, only to flash over SWD with a debug probe.
- `make logs` uses `tools/logs` (its own `.venv`, pinned in `requirements.txt`)
  to read the firmware's USB serial port.
- `rustup component add llvm-tools-preview` (only for `make coverage`)
- Flash command: `make upload-firmware`

## Testing

- Unit tests run on the host (`make test`) through the library target, using
  the mocks in `firmware/src/lib/test_support.rs` (see `docs/adr/adr-0007.md`).
- `make coverage` reports line/region coverage with `llvm-cov`; the library is
  expected to stay at 100%.

## Rationale

Bare-metal Rust gives deterministic timing for servo control and audio
triggers, no runtime overhead, and strong compile-time guarantees — ideal for a
small, reliable, self-contained prop.
