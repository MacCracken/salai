# ADR-001: Migrate editor primitives to muharrir

**Status**: Accepted
**Date**: 2026-03-23

## Context

Salai originally implemented its own `expr`, `hw`, and `history` modules wrapping abaco, ai-hwaccel, and libro respectively. These modules are domain-agnostic editor primitives useful to other AGNOS creative apps (rasa, tazama, shruti).

Analysis of the four editors revealed identical patterns: undo/redo stacks, expression evaluation in numeric fields, hardware-adaptive rendering, hierarchy trees, property inspectors.

## Decision

Extract domain-agnostic editor primitives into **muharrir** (Arabic: editor), a shared crate published to crates.io. Salai depends on `muharrir` and re-exports its types, keeping the same public API surface.

Modules migrated:
- `expr` — math expression evaluation (abaco)
- `hw` — hardware detection and quality tiers (ai-hwaccel)
- `history` — undo/redo via libro audit chain

Modules kept in salai (game-specific):
- `hierarchy` — uses kiran's World, Entity, Parent, Children
- `inspector` — reads kiran ECS components
- `personality` — wraps bhava for NPC authoring
- `viewport` — uses kiran's Camera, OrbitController
- `scene_edit` — reads/writes kiran scene data

## Consequences

- Salai gets muharrir improvements (thread-local evaluator, Cow<str> actions, O(N) hierarchy, command pattern) for free
- 3 fewer direct ecosystem dependencies to maintain
- Future muharrir features (notifications, selection tracking) available automatically
- Benchmark regression risk mitigated — verified no regression post-migration
- Bench file renamed to `editor_benchmarks` to avoid name collision with muharrir's `benchmarks`
