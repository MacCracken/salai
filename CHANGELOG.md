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
- Expanded test suite to 106 tests (93 unit + 13 integration).
- 22 criterion benchmarks covering all modules.

### Changed
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
