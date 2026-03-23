# Salai

> **Salai** — game editor for the [Kiran](https://github.com/MacCracken/kiran) engine

Visual editor for creating game scenes, inspecting entities, and previewing gameplay. Built with egui, powered by kiran + soorat.

## Architecture

```
salai (editor)
  ├── editor      — app state, play/pause/step, scene loading
  ├── inspector   — entity component viewer/editor
  ├── hierarchy   — parent-child scene tree
  └── viewport    — camera control, gizmo modes, grid
```

## Dependencies

- **kiran** — game engine (ECS, scene, input, physics, audio)
- **soorat** — GPU rendering (wgpu)
- **egui/eframe** — immediate-mode UI
- **hisab** — math types

## Usage

```sh
salai                    # Launch empty editor
salai level.toml         # Open a scene file
```

## License

GPL-3.0 — see [LICENSE](LICENSE).
