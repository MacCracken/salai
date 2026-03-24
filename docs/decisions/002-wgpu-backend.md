# ADR-002: Switch eframe from glow to wgpu backend

**Status**: Accepted
**Date**: 2026-03-23

## Context

Salai's viewport needs to display 3D scene rendering from soorat, which uses wgpu for GPU access. The initial eframe configuration used the glow (OpenGL) backend, creating a backend mismatch — soorat's wgpu textures cannot be shared with glow.

## Decision

Switch eframe features from `glow` to `wgpu`:

```toml
eframe = { version = "0.31", features = ["default_fonts", "wgpu", "persistence"] }
```

This allows future texture sharing between soorat's `RenderTarget` and egui's wgpu renderer without CPU readback.

## Consequences

- Enables zero-copy viewport rendering from soorat in V1.0
- wgpu backend may have different platform compatibility vs glow (wider GPU support but requires Vulkan/Metal/DX12)
- Slightly larger binary due to wgpu shader compilation
- No functional regression — all tests pass on both backends
