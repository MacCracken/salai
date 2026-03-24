# Salai

> **Salai** (Arabic: سالاي — apprentice/helper) — game editor for the [Kiran](https://github.com/MacCracken/kiran) engine

Visual editor for creating game scenes, inspecting entities, and previewing gameplay. Built with egui, powered by kiran + soorat + muharrir.

## Features

- **Scene editing** — add/remove entities, edit components, save/load TOML scenes
- **Inspector panel** — view and edit Name, Position, Light, Tags, Material components
- **Hierarchy panel** — collapsible parent-child entity tree with click-to-select
- **Viewport** — interactive 2D viewport with orbit camera, grid overlay, gizmo modes
- **Undo/redo** — tamper-evident history via muharrir (libro audit chain)
- **Expression input** — type `2*pi`, `sin(45)`, `sqrt(2)/2` into numeric fields
- **Hardware detection** — auto-configure quality based on GPU capabilities
- **NPC authoring** — personality trait editing (15 dimensions) and mood vectors via bhava
- **Asset preview** — texture thumbnails (ranga), audio waveforms + loudness (dhvani)
- **Toolbar** — play/pause/step, gizmo mode selector, entity creation
- **Menu bar** — File (open/save), Edit (undo/redo), View (panel toggles)

## Architecture

```
salai
├── editor.rs        — EditorApp, PlayState, EditorState, entity tracking
├── ui/              — egui/eframe interface
│   ├── mod.rs       — SalaiApp (eframe::App), panel layout
│   ├── menu.rs      — File/Edit/View menus
│   ├── toolbar.rs   — play controls, gizmo selector, + Entity
│   ├── hierarchy_panel.rs  — collapsible entity tree
│   ├── inspector_panel.rs  — component display + expr_field
│   └── viewport_panel.rs   — orbit camera, grid, gizmo indicator
├── scene_edit.rs    — entity CRUD, component editing, scene save (all with undo)
├── hierarchy.rs     — kiran-specific parent-child tree builder
├── inspector.rs     — ECS component inspection
├── viewport.rs      — camera, orbit controller, gizmo modes
├── personality.rs   — NPC personality/emotion editing (bhava)
├── texture.rs       — thumbnail generation, color analysis (ranga)
└── audio.rs         — audio inspection, waveform, loudness (dhvani)
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| **kiran** | Game engine — ECS, scenes, input, physics, audio |
| **soorat** | GPU rendering — wgpu |
| **muharrir** | Shared editor primitives — undo/redo, expr eval, hardware detection |
| **bhava** | NPC personality/emotion engine |
| **ranga** | Image processing — thumbnails, color analysis |
| **dhvani** | Audio engine — waveforms, loudness, playback |
| **egui/eframe** | Immediate-mode UI (wgpu backend) |
| **hisab** | Math types (Vec3, etc.) |

## Usage

```sh
salai                    # Launch editor with empty scene
salai level.toml         # Open a scene file
```

## Development

```sh
make check               # fmt + clippy + test + audit
make bench               # criterion benchmarks with CSV history
make coverage            # HTML coverage report
make doc                 # generate documentation
```

## License

GPL-3.0 — see [LICENSE](LICENSE).
