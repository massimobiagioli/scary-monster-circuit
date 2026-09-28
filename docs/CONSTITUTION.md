# Constitution

High-level charter for the Scary Monster Circuit. This document defines *what*
the project is and the non-negotiable boundaries it must respect. It stays
intentionally abstract; implementation details belong in the other docs.

## Mission

Build a Halloween prop: a monster head that tracks nearby movement and, when a
person gets close, triggers a scary song to startle them.

## Scope

- A movable head that follows detected motion.
- Proximity detection that triggers audio playback.
- A self-contained embedded device that runs on its own power.

## Non-Goals

- Cloud connectivity or remote control.
- Video capture, recording, or storage of people.
- Any form of facial recognition or personal data collection.

## Core Behaviors

1. **Idle** — the head rests and watches for motion.
2. **Track** — the head follows movement within a safe range.
3. **Startle** — when someone comes close, playback starts and the prop reacts.

## Constraints

- Runs on a single low-power microcontroller board.
- Operates autonomously with no external computer required.
- Safe to be around people and pets: no sharp moving parts, no hazardous heat.
- Behavior must degrade gracefully (e.g. return to Idle) if a sensor fails.

## Success Criteria

- A person walking past causes the head to follow them.
- A person getting close reliably triggers the scary song.
- The prop runs unattended for a full evening without intervention.

## Governance

- Amendments to this constitution require an explicit, reviewed change.
- All documentation under `docs/` is written in English (see `AGENTS.md`).
