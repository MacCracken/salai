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
- **asset_browser** panel — scan directories, filter by type (IMG/SND/SCN), select and preview assets.
- **console_panel** — ring-buffer log viewer with severity filtering (DBG/INF/WRN/ERR), auto-scroll, clear.
- **profiler_panel** — FPS counter, frame time graph with 60fps target line, entity/history stats.
- **Multi-selection** — `select_add()` (shift-click), `select_toggle()` (ctrl-click), `selected_all()`, `is_selected()`, `selection_count()`. Hierarchy panel wired for shift/ctrl multi-select.
- **personality_panel** — NPC personality editing UI with bhava integration.
  - 15 trait sliders grouped by Social/Cognitive/Behavioral/Professional.
  - Mood vector display with 6 emotion bars (joy, arousal, dominance, trust, interest, frustration).
  - Summary bar showing active trait count and mood label.
  - Color-coded trait levels (Lowest→Highest).
- **picking** module — entity selection via raycasting in the viewport.
  - `pick_entity()` casts ray through click point, tests sphere intersection against entity positions.
  - `pixel_to_ndc()` coordinate conversion, `PickResult` with entity/distance/position.
  - Viewport click wired to picking with shift/ctrl multi-select support.
- **Component add/remove** in inspector — "+ Component" dropdown for Position, Light, Tags, Material with undo/redo.
  - `add_component()`, `remove_component()` in scene_edit, `COMPONENT_TYPES` constant.
- **Prefab creation** — Scene > Create Prefab from Selection menu extracts entity to `PrefabDef`.
  - `extract_prefab()` in scene_edit.
- **File dialogs** — Ctrl+O opens native file dialog for `.toml` scenes, Ctrl+S saves (with Save As dialog if no path).
  - Uses `rfd` crate for cross-platform native dialogs.
- **Keyboard shortcuts** — Delete (despawn selected), Ctrl+Z (undo), Ctrl+Y/Ctrl+Shift+Z (redo), Ctrl+S (save), Ctrl+O (open).
  - All actions logged to console panel.
- **viewport_renderer** module — soorat 3D rendering bridge for egui viewport.
  - `build_grid_lines()` — ground-plane grid with color-coded X/Z axes.
  - `build_gizmo_lines()` — translate arrows, rotate rings, scale handles (all three modes).
  - `camera_view_proj()` — compute view-projection matrix from kiran Camera.
  - `collect_entity_visuals()` — gather entity positions with selection highlighting.
  - Uses soorat's `draw_into_pass()`, `egui_bridge`, and `primitives` APIs.
- 36 criterion benchmarks covering all modules (including scene_edit).

### Changed
- **Migrated to muharrir** — `expr`, `hw`, `history` modules now re-exported from `muharrir` shared editor library instead of local copies. Removes direct deps on abaco, ai-hwaccel, libro.
- `EditorState` selection refactored from `Option<u64>` to `Vec<u64>` for multi-selection support.
- `EditorState::selected_entity` replaced by `select()`, `select_add()`, `select_toggle()`, `selected()`, `selected_all()`, `is_selected()` API.
- `ViewportState::default()` now delegates to `OrbitController::apply()` instead of duplicating orbit math.
- `ComponentInfo` now derives `PartialEq, Eq`.
- `lib.rs` re-exports all key types from submodules.
- eframe backend switched from glow to wgpu for soorat GPU texture sharing.

## [0.1.0] - 2026-03-23

### Added
- EditorApp with play/pause/step state machine.
- Entity inspector — gathers component info (Name, Position, Light, Tags, Material).
- Scene hierarchy builder — parent-child tree with depth-first flattening.
- ViewportState with OrbitController, gizmo modes (translate/rotate/scale), grid toggle.
- CLI: `salai [scene.toml]`.
