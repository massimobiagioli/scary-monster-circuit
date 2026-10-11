# Architecture Decision Records

Registry of every architecture decision record (ADR) for the Scary Monster
Circuit. One file per decision, named `adr-<adr-id>.md`. Keep this table in sync
whenever an ADR is added or changed.

| ID | Title | Status | Date |
| --- | --- | --- | --- |
| [ADR-0001](adr-0001.md) | Firmware layout and behavior state machine | Accepted | 2026-09-27 |
| [ADR-0002](adr-0002.md) | Two side-mounted distance sensors for tracking | Accepted | 2026-09-27 |
| [ADR-0003](adr-0003.md) | Audio playback through a UART MP3 player | Accepted | 2026-09-27 |
| [ADR-0004](adr-0004.md) | Library/binary split for host-testable logic | Accepted | 2026-09-27 |
| [ADR-0005](adr-0005.md) | Markdown-to-PDF tool with Mermaid and a Python venv | Accepted | 2026-09-27 |
| [ADR-0006](adr-0006.md) | Breadboard-style pictures for the assembly guide | Accepted | 2026-09-27 |
| [ADR-0007](adr-0007.md) | Mock-based unit tests for the whole library | Accepted | 2026-09-27 |
| [ADR-0008](adr-0008.md) | Live diagnostic probe over RTT | Superseded by [ADR-0011](adr-0011.md) | 2026-10-11 |
| [ADR-0009](adr-0009.md) | Project-local, pinned probe-rs tool | Accepted | 2026-10-11 |
| [ADR-0010](adr-0010.md) | Flash over USB without a debug probe | Accepted | 2026-10-11 |
| [ADR-0011](adr-0011.md) | Logs over USB serial instead of RTT | Accepted | 2026-10-11 |
| [ADR-0012](adr-0012.md) | Bootable flash image and reliable USB enumeration | Accepted | 2026-10-11 |
| [ADR-0013](adr-0013.md) | Post-action hold after the scary song | Accepted | 2026-10-11 |
| [ADR-0014](adr-0014.md) | Always stream the live dashboard | Accepted | 2026-10-11 |
| [ADR-0015](adr-0015.md) | Live log with a one-time header and one-line rows | Accepted | 2026-10-11 |
| [ADR-0016](adr-0016.md) | Dedicated serial log reader with a clean Ctrl-C | Accepted | 2026-10-11 |
| [ADR-0017](adr-0017.md) | Recalibrated thresholds and honest mp3 column | Accepted | 2026-10-11 |
| [ADR-0018](adr-0018.md) | The servo only moves when its angle changes | Accepted | 2026-10-11 |
| [ADR-0019](adr-0019.md) | The servo moves only when the behavior state changes | Accepted | 2026-10-11 |
| [ADR-0020](adr-0020.md) | Follow from the nearest sensor within 30 cm | Accepted | 2026-10-11 |
