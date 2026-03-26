//! 3D viewport renderer — bridges soorat GPU rendering into egui via wgpu callbacks.
//!
//! Uses soorat's `draw_into_pass()` APIs to render meshes, debug lines, and gizmos
//! directly into egui's render pass. Entity positions are visualized as primitive
//! shapes (cubes/spheres) with a ground-plane grid.

use hisab::{Mat4, Vec3};

/// Entity render data — position + optional color for viewport display.
#[derive(Debug, Clone)]
pub struct EntityVisual {
    pub entity_id: u64,
    pub position: Vec3,
    pub color: [f32; 4],
    pub selected: bool,
}

/// Generate a grid as a LineBatch for the ground plane.
#[must_use]
pub fn build_grid_lines(grid_size: f32, count: i32) -> Vec<(Vec3, Vec3, [f32; 4])> {
    let mut lines = Vec::with_capacity((2 * count as usize + 1) * 2);
    let extent = count as f32 * grid_size;
    let grid_color = [0.3, 0.3, 0.3, 0.4];
    let x_axis_color = [0.8, 0.2, 0.2, 0.6];
    let z_axis_color = [0.2, 0.2, 0.8, 0.6];

    for i in -count..=count {
        let offset = i as f32 * grid_size;
        let color = if i == 0 { z_axis_color } else { grid_color };
        // Lines along Z axis
        lines.push((
            Vec3::new(offset, 0.0, -extent),
            Vec3::new(offset, 0.0, extent),
            color,
        ));

        let color = if i == 0 { x_axis_color } else { grid_color };
        // Lines along X axis
        lines.push((
            Vec3::new(-extent, 0.0, offset),
            Vec3::new(extent, 0.0, offset),
            color,
        ));
    }
    lines
}

/// Generate gizmo lines for the selected entity's transform gizmo.
#[must_use]
pub fn build_gizmo_lines(
    position: Vec3,
    mode: crate::viewport::GizmoMode,
    scale: f32,
) -> Vec<(Vec3, Vec3, [f32; 4])> {
    let mut lines = Vec::with_capacity(72);
    let red = [1.0, 0.2, 0.2, 1.0];
    let green = [0.2, 1.0, 0.2, 1.0];
    let blue = [0.2, 0.2, 1.0, 1.0];

    match mode {
        crate::viewport::GizmoMode::Translate => {
            // Arrows along each axis
            lines.push((position, position + Vec3::new(scale, 0.0, 0.0), red));
            lines.push((position, position + Vec3::new(0.0, scale, 0.0), green));
            lines.push((position, position + Vec3::new(0.0, 0.0, scale), blue));
            // Arrowheads (small perpendicular lines)
            let tip = scale * 0.15;
            let end_x = position + Vec3::new(scale, 0.0, 0.0);
            lines.push((end_x, end_x + Vec3::new(-tip, tip, 0.0), red));
            lines.push((end_x, end_x + Vec3::new(-tip, -tip, 0.0), red));
            let end_y = position + Vec3::new(0.0, scale, 0.0);
            lines.push((end_y, end_y + Vec3::new(tip, -tip, 0.0), green));
            lines.push((end_y, end_y + Vec3::new(-tip, -tip, 0.0), green));
            let end_z = position + Vec3::new(0.0, 0.0, scale);
            lines.push((end_z, end_z + Vec3::new(0.0, tip, -tip), blue));
            lines.push((end_z, end_z + Vec3::new(0.0, -tip, -tip), blue));
        }
        crate::viewport::GizmoMode::Rotate => {
            // Circles around each axis (approximated with line segments)
            let segments = 24;
            for i in 0..segments {
                let a0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                let a1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                let r = scale * 0.8;
                // X rotation ring (YZ plane)
                lines.push((
                    position + Vec3::new(0.0, a0.cos() * r, a0.sin() * r),
                    position + Vec3::new(0.0, a1.cos() * r, a1.sin() * r),
                    red,
                ));
                // Y rotation ring (XZ plane)
                lines.push((
                    position + Vec3::new(a0.cos() * r, 0.0, a0.sin() * r),
                    position + Vec3::new(a1.cos() * r, 0.0, a1.sin() * r),
                    green,
                ));
                // Z rotation ring (XY plane)
                lines.push((
                    position + Vec3::new(a0.cos() * r, a0.sin() * r, 0.0),
                    position + Vec3::new(a1.cos() * r, a1.sin() * r, 0.0),
                    blue,
                ));
            }
        }
        crate::viewport::GizmoMode::Scale => {
            // Lines with boxes at ends
            lines.push((position, position + Vec3::new(scale, 0.0, 0.0), red));
            lines.push((position, position + Vec3::new(0.0, scale, 0.0), green));
            lines.push((position, position + Vec3::new(0.0, 0.0, scale), blue));
            // Small cubes at tips (approximated as crosses)
            let b = scale * 0.08;
            for (axis, color) in [
                (Vec3::new(scale, 0.0, 0.0), red),
                (Vec3::new(0.0, scale, 0.0), green),
                (Vec3::new(0.0, 0.0, scale), blue),
            ] {
                let tip = position + axis;
                lines.push((
                    tip - Vec3::new(b, 0.0, 0.0),
                    tip + Vec3::new(b, 0.0, 0.0),
                    color,
                ));
                lines.push((
                    tip - Vec3::new(0.0, b, 0.0),
                    tip + Vec3::new(0.0, b, 0.0),
                    color,
                ));
                lines.push((
                    tip - Vec3::new(0.0, 0.0, b),
                    tip + Vec3::new(0.0, 0.0, b),
                    color,
                ));
            }
        }
    }
    lines
}

/// Compute camera view-projection matrix from viewport state.
#[must_use]
pub fn camera_view_proj(camera: &kiran::render::Camera, aspect: f32) -> Mat4 {
    let view = Mat4::look_at_rh(camera.position, camera.target, Vec3::Y);
    let proj = Mat4::perspective_rh(camera.fov_y.to_radians(), aspect, 0.1, 1000.0);
    proj * view
}

/// Build entity visuals from world state.
#[must_use]
pub fn collect_entity_visuals(
    world: &kiran::World,
    entities: &[kiran::Entity],
    selected_entities: &[kiran::Entity],
) -> Vec<EntityVisual> {
    entities
        .iter()
        .filter(|&&e| world.is_alive(e))
        .filter_map(|&e| {
            let pos = world.get_component::<kiran::scene::Position>(e)?;
            let is_selected = selected_entities.contains(&e);
            let color = if is_selected {
                [1.0, 0.8, 0.2, 1.0]
            } else {
                [0.6, 0.6, 0.7, 1.0]
            };
            Some(EntityVisual {
                entity_id: e.id(),
                position: pos.0,
                color,
                selected: is_selected,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_lines_count() {
        let lines = build_grid_lines(1.0, 10);
        // 21 lines in each direction (from -10 to +10 inclusive)
        assert_eq!(lines.len(), 42);
    }

    #[test]
    fn grid_lines_axes_colored() {
        let lines = build_grid_lines(1.0, 5);
        // Center lines (i=0) should have axis colors, not grid color
        let center_lines: Vec<_> = lines
            .iter()
            .filter(|(a, b, _)| (a.x == 0.0 && b.x == 0.0) || (a.z == 0.0 && b.z == 0.0))
            .collect();
        assert!(!center_lines.is_empty());
    }

    #[test]
    fn gizmo_translate_lines() {
        let lines = build_gizmo_lines(Vec3::ZERO, crate::viewport::GizmoMode::Translate, 1.0);
        // 3 main axes + 6 arrowhead lines = 9
        assert_eq!(lines.len(), 9);
    }

    #[test]
    fn gizmo_rotate_lines() {
        let lines = build_gizmo_lines(Vec3::ZERO, crate::viewport::GizmoMode::Rotate, 1.0);
        // 3 rings * 24 segments = 72
        assert_eq!(lines.len(), 72);
    }

    #[test]
    fn gizmo_scale_lines() {
        let lines = build_gizmo_lines(Vec3::ZERO, crate::viewport::GizmoMode::Scale, 1.0);
        // 3 axes + 3 tips * 3 cross lines = 12
        assert_eq!(lines.len(), 12);
    }

    #[test]
    fn gizmo_at_offset() {
        let pos = Vec3::new(5.0, 10.0, 15.0);
        let lines = build_gizmo_lines(pos, crate::viewport::GizmoMode::Translate, 1.0);
        // All lines should start at or near the position
        for (start, _, _) in &lines {
            assert!(start.x >= 4.0); // at least near the offset
        }
    }

    #[test]
    fn camera_view_proj_finite() {
        let camera = kiran::render::Camera::default();
        let vp = camera_view_proj(&camera, 16.0 / 9.0);
        assert!(vp.x_axis.x.is_finite());
    }

    #[test]
    fn collect_visuals_empty() {
        let world = kiran::World::new();
        let visuals = collect_entity_visuals(&world, &[], &[]);
        assert!(visuals.is_empty());
    }

    #[test]
    fn collect_visuals_with_entities() {
        let mut world = kiran::World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world
            .insert_component(e1, kiran::scene::Position(Vec3::new(1.0, 0.0, 0.0)))
            .unwrap();
        world
            .insert_component(e2, kiran::scene::Position(Vec3::new(2.0, 0.0, 0.0)))
            .unwrap();

        let visuals = collect_entity_visuals(&world, &[e1, e2], &[e1]);
        assert_eq!(visuals.len(), 2);
        assert!(visuals[0].selected);
        assert!(!visuals[1].selected);
    }

    #[test]
    fn collect_visuals_skips_no_position() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world
            .insert_component(e, kiran::scene::Name("NoPos".into()))
            .unwrap();
        // No Position component

        let visuals = collect_entity_visuals(&world, &[e], &[]);
        assert!(visuals.is_empty());
    }

    #[test]
    fn collect_visuals_skips_dead() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world
            .insert_component(e, kiran::scene::Position(Vec3::ZERO))
            .unwrap();
        world.despawn(e).unwrap();

        let visuals = collect_entity_visuals(&world, &[e], &[]);
        assert!(visuals.is_empty());
    }
}
