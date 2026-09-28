# Principles

Guiding engineering principles for the Scary Monster Circuit. Every change
should be checked against these.

## 1. State Machine First

Model the behavior as an explicit finite state machine (FSM). No hidden modes,
no scattered flags.

- States: `Idle`, `Track`, `Startle`, plus a `Fault` fallback.
- Every transition has an explicit trigger (sensor event, timeout, etc.).
- All state lives in one place; the main loop is a dispatcher, not a decision
  tree.

## 2. Simple, No Overengineering

Prefer the smallest solution that solves the problem. Complexity must be
justified by a real requirement, never by a hypothetical one.

- Small, single-purpose functions with clear names.
- No clever abstractions; favor the obvious solution.
- No premature generalization: do not build frameworks, traits, or config for
  a single use case.
- Apply YAGNI — do not add features, hooks, or options "just in case".
- If a piece of code is hard to explain, simplify it instead of documenting it.

## 3. Modular by Component

Structure the firmware as independent modules, one per component.

- One module per component: sensor, servo, audio, state machine.
- Each module exposes a small, clear interface and hides its internals.
- Hardware access is isolated behind thin wrappers so logic stays testable.
- Modules can be read, changed, and tested in isolation; no hidden coupling.

## 4. Use Libraries for Each Component

Do not reinvent what the ecosystem already provides well.

- Use `embedded-hal` traits and established crates to drive each component
  (servo/PWM, distance sensor, audio) instead of hand-rolling drivers.
- Prefer well-maintained, widely used crates over custom implementations.
- Pin dependency versions for reproducible builds.
- Keep the dependency set small and justified: every crate must earn its place
  (this complements "No Overengineering").

## 5. Easy to Read

- Code reads top-to-bottom like prose.
- Names describe intent, not implementation.
- No comments explaining *what* — only *why* when non-obvious.
- Consistent formatting (`cargo fmt`) and lint-clean (`cargo clippy`).

## 6. Well-Documented Schematics

- Every circuit diagram is included and kept up to date.
- Each connection documents pin, signal, and voltage.
- Diagrams live alongside the code that drives them.
- A wiring table accompanies each schematic for quick reference.

## 7. Educational

- The project should teach embedded Rust and hardware basics.
- Document decisions and trade-offs, not just results.
- Prefer readable examples over terse tricks.

## 8. Easy to Build

- One-command build and flash.
- Reproducible toolchain and dependency versions.
- A newcomer can go from zero to running prop by following the docs.
- No manual steps that cannot be scripted.

## 9. Safety and Graceful Failure

- Never drive actuators beyond safe mechanical limits.
- On sensor or audio failure, fall back to `Fault` or `Idle`, never hang.
- Hardware failures must not produce erratic, unsafe motion.
