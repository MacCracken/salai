# Salai Architecture

## Overview

Salai is a visual game editor for the Kiran engine. It provides an egui-based GUI for building game scenes, inspecting entities, editing components, and previewing gameplay.

## Module Map

```
salai
├── editor.rs          — EditorApp state machine, entity tracking, scene loading
├── ui/                — egui/eframe editor interface
│   ├── mod.rs         — SalaiApp (eframe::App impl), panel layout orchestration
│   ├── menu.rs        — File (open/save), Edit (undo/redo), View (panel toggles)
│   ├── toolbar.rs     — play/pause/step, gizmo mode, + Entity button
│   ├── hierarchy_panel.rs  — collapsible entity tree, click-to-select
│   ├── inspector_panel.rs  — component display, expr_field widget
│   └── viewport_panel.rs   — orbit camera, grid overlay, gizmo indicator
├── scene_edit.rs      — entity CRUD + component editing with undo/redo
├── hierarchy.rs       — kiran-specific parent-child tree (World, Entity, Parent)
├── inspector.rs       — ECS component inspection (Name, Position, Light, Tags, Material)
├── viewport.rs        — ViewportState, OrbitController, GizmoMode
├── personality.rs     — NPC personality editing via bhava
├── texture.rs         — texture thumbnails and analysis via ranga
├── audio.rs           — audio preview and analysis via dhvani
└── lib.rs             — crate root, re-exports muharrir primitives
```

## Dependency Graph

```
salai
├── kiran (game engine)
│   └── hisab (math)
├── soorat (GPU rendering)
│   └── wgpu
├── muharrir (editor primitives, from crates.io)
│   ├── libro (undo/redo audit chain)
│   ├── abaco (expression evaluation)
│   └── ai-hwaccel (hardware detection)
├── bhava (personality/emotion)
├── ranga (image processing)
├── dhvani (audio engine)
├── egui + eframe (UI, wgpu backend)
└── toml (scene serialization)
```

## Data Flow

```
User Input (mouse/keyboard)
    │
    ├─→ Toolbar ──→ EditorApp.toggle_play() / step_frame()
    ├─→ Menu ────→ save_scene() / History.undo() / History.redo()
    ├─→ Hierarchy ─→ EditorState.select(entity)
    ├─→ Inspector ──→ (read-only display, V0.4+ editable)
    └─→ Viewport ──→ ViewportState.rotate() / zoom()

Scene Editing (scene_edit module)
    │
    ├─→ add_entity() ──→ World.spawn() + tracked_entities + History.record()
    ├─→ set_position() → World.insert_component() + History.record()
    ├─→ set_name() ────→ World.insert_component() + History.record()
    └─→ extract_scene() → SceneDefinition → TOML file

Rendering
    │
    └─→ viewport_panel → egui Painter (2D overlay)
        └─→ (V1.0: soorat RenderTarget → egui wgpu texture)
```

## Key Design Decisions

1. **muharrir for shared primitives** — expr, hw, history modules come from the shared editor library rather than local implementations. Keeps salai focused on game-specific logic.

2. **Entity tracking** — kiran's World doesn't expose an entity iterator, so EditorApp maintains a `tracked_entities` list updated on spawn/despawn/load.

3. **wgpu backend** — eframe configured with wgpu (not glow) for future soorat texture sharing in the viewport.

4. **History via libro** — undo/redo uses libro's tamper-evident audit chain through muharrir, providing verifiable action history.

5. **Scene round-trip** — `extract_scene()` reconstructs `SceneDefinition` from live ECS state, enabling save-to-TOML without a separate document model.
