# AGENTS.md

## Project

Scary Monster Circuit — a Halloween prop with a head that follows movement and
plays a scary song when someone gets close. See `docs/` for details. Firmware
lives in `firmware/`, schematics in `schematics/`, tools in `tools/`.

## Documentation

All project documentation lives in `docs/` and is the source of truth:

- [docs/CONSTITUTION.md](docs/CONSTITUTION.md) — project charter and high-level scope.
- [docs/STACK.md](docs/STACK.md) — hardware and software stack (bare-metal Rust on Raspberry Pi Pico).
- [docs/PRINCIPLES.md](docs/PRINCIPLES.md) — engineering principles (state machine, maintainability, readability, schematics, education, buildability).
- [docs/adr/index.md](docs/adr/index.md) — registry of all architecture decision records (ADRs).

## Schematics

Schematics live in `schematics/`. The reference document uses Markdown with
Mermaid diagrams; the assembly guide uses **breadboard pictures** drawn by
`tools/breadboard`. `make schematics` draws the pictures and regenerates every
PDF with the reusable `tools/md2pdf` tool.

- **Every schematic MUST be understandable by a 12-year-old** (the assembly
  guide is written for a 10-year-old).
- The assembly guide MUST look like a LEGO booklet: one picture per step, the
  new part highlighted, earlier parts kept in place.
- Wiring MUST be shown as a breadboard picture, never as a flow chart.
- Label every connection with its pin, and always show a common ground.
- Keep the Markdown source and the generated PDF in sync.

## Skills

Reusable project skills live in `.opencode/skills/` and are loaded on demand via
the `skill` tool:

- `.opencode/skills/embedded-rust/SKILL.md` — bare-metal `no_std` Rust conventions for the RP2040.
- `.opencode/skills/state-machine/SKILL.md` — design and maintain the behavior FSM.
- `.opencode/skills/hardware-schematics/SKILL.md` — document and maintain circuit schematics and wiring.
- `.opencode/skills/firmware-workflow/SKILL.md` — build, flash, and inspect firmware via the Makefile.
- `.opencode/skills/rust-quality/SKILL.md` — run tests, lint, and lint fixes via the Makefile.

## Tooling

The `Makefile` is the single entry point for build tasks (`make help` lists all
targets): `upload-firmware`, `logs`, `test`, `coverage`, `lint`, `lint-fix`,
`schematics`.

- `tools/md2pdf` — reusable Markdown-to-PDF tool. Always use it to build
  schematic PDFs; never convert documents by hand.
- `tools/breadboard` — draws the breadboard-style assembly pictures.

## Rules

- **All files under `docs/` MUST be written in English.**
- **After implementing a feature or a fix, record the decision**: add or update
  an ADR at `docs/adr/adr-<adr-id>.md` and update the registry
  `docs/adr/index.md`.
- **Never install software with Homebrew.** Use project-local tooling instead.
- **Any Python tooling MUST run inside its own `.venv`** (see `docs/STACK.md`).
- Keep docs in sync with the code whenever behavior changes.
- Follow the principles in `docs/PRINCIPLES.md` for all code changes.
- Prefer the relevant skill in `.opencode/skills/` and the `Makefile` targets over ad-hoc commands.
