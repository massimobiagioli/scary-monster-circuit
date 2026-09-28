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
  - `defmt` + `defmt-rtt` — lightweight logging over RTT (optional, dev only).
- **Build/Flash:** `cargo` + `probe-rs` (or `elf2uf2-rs` with BOOTSEL drag-drop).
- **Config:** `memory.x` linker script and `.cargo/config.toml` for the target.

## Tooling (Python)

- `tools/md2pdf` converts Markdown with Mermaid diagrams into a PDF.
- **Every Python tool MUST run inside its own `.venv`.** Never install Python
  packages into the system interpreter.
- Dependencies are pinned in a `requirements.txt` next to each tool, and the
  `.venv` is created automatically on first use (and is git-ignored).

## Development Environment

- `rustup target add thumbv6m-none-eabi`
- `cargo install probe-rs-tools` (or `cargo install elf2uf2-rs`)
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
