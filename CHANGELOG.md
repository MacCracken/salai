# Changelog

## [Unreleased]

### Added
- **expr** module — math expression evaluator for inspector property fields, powered by `abaco`.
  - `eval_f64()`, `eval_or()`, `eval_or_parse()` for evaluating user input like `2*pi`, `sin(45)`, `sqrt(2)/2`.
  - Supports arithmetic, trig functions, constants (pi, e, tau), scientific notation.
- **hw** module — hardware capability detection via `ai-hwaccel`.
  - `HardwareProfile::detect()` probes system GPUs/accelerators at startup.
  - `QualityTier` (Low/Medium/High/Ultra) maps hardware to viewport quality settings.
  - `suggested_grid_size()`, `default_debug_shapes()`, `gpu_memory_display()` for auto-configuration.
- **history** module — undo/redo system backed by `libro` audit chain.
  - `History::record()`, `undo()`, `redo()` with cursor-based navigation.
  - `verify()` for tamper-evident integrity checking of action history.
  - `applied_entries()`, `page()` for history display.
- **personality** module — NPC personality and emotion editing via `bhava`.
  - `NpcPersonality` wraps 15-dimension trait profile + mood vector.
  - `inspector_summary()` for panel display, `compatibility()`, `blend()` for NPC authoring.
  - `ALL_TRAITS`, `ALL_LEVELS` constants for building editor UI.
- 28 criterion benchmarks covering all modules.

### Changed
- **Migrated to muharrir** — `expr`, `hw`, `history` modules now re-exported from `muharrir` shared editor library instead of local copies. Removes direct deps on abaco, ai-hwaccel, libro.
- `EditorState::selected_entity` is now private — use `select()`/`selected()` API. Preserves entity generation to prevent stale selection after recycling.
- `ViewportState::default()` now delegates to `OrbitController::apply()` instead of duplicating orbit math.
- `ComponentInfo` now derives `PartialEq, Eq`.
- `lib.rs` re-exports all key types from submodules.

## [0.1.0] - 2026-03-23

### Added
- EditorApp with play/pause/step state machine.
- Entity inspector — gathers component info (Name, Position, Light, Tags, Material).
- Scene hierarchy builder — parent-child tree with depth-first flattening.
- ViewportState with OrbitController, gizmo modes (translate/rotate/scale), grid toggle.
- CLI: `salai [scene.toml]`.
