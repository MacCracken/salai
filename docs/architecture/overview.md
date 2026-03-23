# Salai Architecture

## Overview

Salai is a visual game editor that consumes kiran (engine) and soorat (rendering) as libraries. It provides a GUI for building game scenes, inspecting entities, and previewing gameplay.

## Module Structure

```
src/
├── main.rs       — CLI entry point
├── lib.rs        — crate root
├── editor.rs     — EditorApp, PlayState, EditorState
├── inspector.rs  — component inspection for selected entity
├── hierarchy.rs  — parent-child scene tree builder
└── viewport.rs   — camera, gizmos, grid state
```

## Data Flow

```
salai (editor UI)
  ├── reads/writes → kiran::World (ECS)
  ├── reads → kiran::scene (TOML loading)
  ├── reads → kiran::render (Camera, Renderer)
  └── renders via → soorat (wgpu GPU)
```
