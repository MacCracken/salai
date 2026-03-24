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
- **texture** module — image processing utilities via `ranga`.
  - `generate_thumbnail()` with aspect-preserving resize.
  - `inspect_texture()`, `average_color()`, `luminance_histogram()`, `color_to_hex()`.
- **audio** module — audio asset preview via `dhvani`.
  - `inspect_audio()` / `inspect_audio_with_loudness()` for metadata extraction.
  - `waveform()` for visualization data, `format_duration()`, `amplitude_to_db_str()`.
  - `buffer_from_samples()`, `normalize()` for playback preparation.
- **ui** module — egui/eframe editor interface.
  - `SalaiApp` eframe wrapper with full panel layout.
  - Menu bar (File open/save, Edit undo/redo, View panel toggles).
  - Toolbar (play/pause/step, gizmo mode selector, entity count status).
  - Hierarchy panel with collapsible entity tree and click-to-select.
  - Inspector panel showing selected entity components.
  - Central viewport placeholder (3D rendering in V0.3).
  - `expr_field()` widget for expression-evaluable numeric inputs.
- `EditorApp::spawn_entity()` and `entities()` for tracked entity management.
- **viewport_panel** — interactive 2D viewport with egui painter.
  - Orbit camera: left-drag to rotate, scroll to zoom.
  - Grid overlay with distance-adaptive cell size.
  - Camera direction indicator and origin marker.
  - Gizmo mode indicator (color-coded Move/Rotate/Scale).
  - Camera info overlay (distance, yaw, pitch).
- **scene_edit** module — entity CRUD and scene serialization with undo/redo.
  - `add_entity()`, `set_position()`, `set_name()`, `set_light_intensity()` — all recorded in history.
  - `extract_scene()` — serialize world state back to kiran `SceneDefinition`.
  - `save_scene()` / `scene_to_toml()` — write scene to TOML file.
  - Toolbar "+ Entity" button wired to `add_entity`.
  - File > Save Scene wired to `extract_scene` + `save_scene`.
- `EditorApp::despawn_entity()` with selection cleanup.
- Switched eframe backend from glow to wgpu for soorat compatibility.
- `muharrir` dependency updated from path to crates.io `0.23`.
- 33 criterion benchmarks covering all modules.

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
