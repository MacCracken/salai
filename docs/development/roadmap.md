# Salai Roadmap

> Game editor for the Kiran engine.

## V0.1 — Scaffold (done, 2026-03-23)

- EditorApp with play/pause/step state machine
- Entity inspector (Name, Position, Light, Tags, Material)
- Hierarchy builder with parent-child tree + depth-first flatten
- ViewportState with orbit camera, gizmo modes, grid/debug toggles
- CLI binary with scene loading

## V0.2 — egui Integration

- [ ] eframe event loop wiring
- [ ] Inspector panel with egui widgets (editable fields)
- [ ] Hierarchy panel with collapsible tree
- [ ] Toolbar (play/pause/step, gizmo mode selector)
- [ ] Menu bar (file open/save, view toggles)

## V0.3 — Viewport

- [ ] soorat-based 3D viewport in egui
- [ ] Orbit camera mouse interaction
- [ ] Entity selection by clicking in viewport
- [ ] Gizmo rendering (translate arrows, rotate rings, scale handles)
- [ ] Grid overlay rendering

## V0.4 — Scene Editing

- [ ] Add/remove entities from editor
- [ ] Component drag-and-drop (add physics, add sound, add material)
- [ ] Scene save to TOML
- [ ] Undo/redo stack
- [ ] Prefab creation from selection

## V1.0 — Production

- [ ] Asset browser (textures, models, sounds)
- [ ] Console/log panel
- [ ] Performance profiler panel
- [ ] Multi-selection and group operations
- [ ] Publish to crates.io
