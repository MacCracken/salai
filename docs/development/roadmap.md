# Salai Roadmap

> Game editor for the Kiran engine.

## V0.1 — Scaffold (done, 2026-03-23)

- EditorApp with play/pause/step state machine
- Entity inspector (Name, Position, Light, Tags, Material)
- Hierarchy builder with parent-child tree + depth-first flatten
- ViewportState with orbit camera, gizmo modes, grid/debug toggles
- CLI binary with scene loading

## P0 — Ecosystem Integration (priority)

> Wire in AGNOS crates that reduce future workload across the roadmap.

- [ ] **ranga** — image processing pipeline for texture/asset previews and viewport overlays
- [ ] **dhvani** — audio engine hookup for play-mode sound and audio asset preview
- [x] **abaco** — expression evaluator in inspector property fields (e.g. `2*pi`, unit math)
- [x] **ai-hwaccel** — detect GPU capabilities, auto-configure viewport quality/feature set
- [x] **libro** — audit-chain backend for undo/redo history (feeds into V0.4)
- [x] **bhava** — emotion/personality editing panel for NPC/character authoring

## V0.2 — egui Integration

- [ ] eframe event loop wiring
- [ ] Inspector panel with egui widgets (editable fields)
  - [ ] abaco-powered expression input for numeric properties
- [ ] Hierarchy panel with collapsible tree
- [ ] Toolbar (play/pause/step, gizmo mode selector)
- [ ] Menu bar (file open/save, view toggles)

## V0.3 — Viewport

- [ ] soorat-based 3D viewport in egui
- [ ] ai-hwaccel capability detection → adaptive render quality
- [ ] Orbit camera mouse interaction
- [ ] Entity selection by clicking in viewport
- [ ] Gizmo rendering (translate arrows, rotate rings, scale handles)
- [ ] Grid overlay rendering

## V0.4 — Scene Editing

- [ ] Add/remove entities from editor
- [ ] Component drag-and-drop (add physics, add sound, add material)
- [ ] Scene save to TOML
- [ ] Undo/redo stack (libro audit-chain backend)
- [ ] Prefab creation from selection
- [ ] bhava personality/emotion editing for NPC entities

## V1.0 — Production

- [ ] Asset browser (textures, models, sounds)
  - [ ] ranga-powered texture thumbnails and preview
  - [ ] dhvani-powered audio waveform preview and playback
- [ ] Console/log panel
- [ ] Performance profiler panel
- [ ] Multi-selection and group operations
- [ ] Publish to crates.io
