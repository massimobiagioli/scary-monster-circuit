---
name: state-machine
description: Design and maintain the Scary Monster Circuit finite state machine (Idle, Track, Startle, Fault). Use when changing behavior, adding transitions, or wiring sensor and actuator events into the main loop.
license: MIT
compatibility: opencode
metadata:
  audience: firmware-developers
  principle: state-machine-first
---

## What I do

- Model all behavior as one explicit finite state machine.
- Define states, triggers, and transitions in a single place.
- Keep the main loop a dispatcher, not a decision tree.

## When to use me

Use me whenever behavior changes: a new sensor event, a new reaction, a
timeout, or anything that alters what the prop does and when.

## States

- `Idle` — rest, watch for motion.
- `Track` — follow movement within a safe range.
- `Startle` — person is close; trigger the scary song and react.
- `Fault` — a sensor or actuator failed; degrade safely.

## Rules

- Every transition has exactly one explicit trigger (event or timeout).
- All state lives in one enum; no scattered boolean flags.
- On any hardware failure, transition to `Fault` or `Idle` — never hang.
- Clamp servo motion to safe limits before any actuation.
- Transitions must be exhaustive (`match` with no catch-all hiding states).

## Checklist

1. Draw the transition table (state -> trigger -> next state).
2. Update the FSM enum and dispatch `match`.
3. Verify every state is reachable and every trigger is handled.
4. Run `make lint` and `make test`.
