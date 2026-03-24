//! Viewport state — camera control and gizmo management for the editor view.

use kiran::render::{Camera, OrbitController};
use serde::{Deserialize, Serialize};

/// Gizmo mode for transform manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

/// Viewport state — manages the editor camera and gizmo settings.
#[derive(Debug, Clone)]
pub struct ViewportState {
    pub camera: Camera,
    pub orbit: OrbitController,
    pub gizmo_mode: GizmoMode,
    pub show_grid: bool,
    pub show_debug_shapes: bool,
    pub grid_size: f32,
}

impl Default for ViewportState {
    fn default() -> Self {
        let mut camera = Camera::default();
        let orbit = OrbitController::default();
        orbit.apply(&mut camera);

        Self {
            camera,
            orbit,
            gizmo_mode: GizmoMode::Translate,
            show_grid: true,
            show_debug_shapes: true,
            grid_size: 1.0,
        }
    }
}

impl ViewportState {
    /// Rotate the viewport camera by mouse delta.
    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.orbit.rotate(dx * 0.01, dy * 0.01);
        self.orbit.apply(&mut self.camera);
    }

    /// Zoom the viewport camera.
    pub fn zoom(&mut self, delta: f32) {
        self.orbit.zoom(delta);
        self.orbit.apply(&mut self.camera);
    }

    /// Cycle through gizmo modes: Translate → Rotate → Scale → Translate.
    pub fn cycle_gizmo(&mut self) {
        self.gizmo_mode = match self.gizmo_mode {
            GizmoMode::Translate => GizmoMode::Rotate,
            GizmoMode::Rotate => GizmoMode::Scale,
            GizmoMode::Scale => GizmoMode::Translate,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_default() {
        let vp = ViewportState::default();
        assert_eq!(vp.gizmo_mode, GizmoMode::Translate);
        assert!(vp.show_grid);
        assert!(vp.show_debug_shapes);
        assert_eq!(vp.grid_size, 1.0);
    }

    #[test]
    fn viewport_rotate() {
        let mut vp = ViewportState::default();
        let start_yaw = vp.orbit.yaw;
        vp.rotate(100.0, 0.0);
        assert_ne!(vp.orbit.yaw, start_yaw);
    }

    #[test]
    fn viewport_zoom() {
        let mut vp = ViewportState::default();
        let start = vp.orbit.distance;
        vp.zoom(2.0);
        assert!(vp.orbit.distance < start);
    }

    #[test]
    fn viewport_cycle_gizmo() {
        let mut vp = ViewportState::default();
        assert_eq!(vp.gizmo_mode, GizmoMode::Translate);

        vp.cycle_gizmo();
        assert_eq!(vp.gizmo_mode, GizmoMode::Rotate);

        vp.cycle_gizmo();
        assert_eq!(vp.gizmo_mode, GizmoMode::Scale);

        vp.cycle_gizmo();
        assert_eq!(vp.gizmo_mode, GizmoMode::Translate);
    }

    #[test]
    fn gizmo_mode_serde() {
        let mode = GizmoMode::Rotate;
        let json = serde_json::to_string(&mode).unwrap();
        let decoded: GizmoMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, decoded);
    }

    #[test]
    fn viewport_zoom_extreme() {
        let mut vp = ViewportState::default();
        vp.zoom(10000.0); // zoom way in
        assert_eq!(vp.orbit.distance, vp.orbit.min_distance);

        vp.zoom(-10000.0); // zoom way out
        assert_eq!(vp.orbit.distance, vp.orbit.max_distance);
    }

    #[test]
    fn viewport_rotate_many_times() {
        let mut vp = ViewportState::default();
        for _ in 0..1000 {
            vp.rotate(1.0, 0.5);
        }
        // Should not panic or produce NaN
        assert!(vp.orbit.yaw.is_finite());
        assert!(vp.orbit.pitch.is_finite());
        assert!(vp.camera.position.x.is_finite());
    }

    #[test]
    fn viewport_grid_toggle() {
        let mut vp = ViewportState::default();
        assert!(vp.show_grid);
        vp.show_grid = false;
        assert!(!vp.show_grid);
    }

    #[test]
    fn viewport_debug_shapes_toggle() {
        let mut vp = ViewportState::default();
        assert!(vp.show_debug_shapes);
        vp.show_debug_shapes = false;
        assert!(!vp.show_debug_shapes);
    }

    #[test]
    fn viewport_grid_size() {
        let mut vp = ViewportState::default();
        assert_eq!(vp.grid_size, 1.0);
        vp.grid_size = 0.5;
        assert_eq!(vp.grid_size, 0.5);
    }

    #[test]
    fn gizmo_mode_all_variants_serde() {
        for mode in [GizmoMode::Translate, GizmoMode::Rotate, GizmoMode::Scale] {
            let json = serde_json::to_string(&mode).unwrap();
            let decoded: GizmoMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, decoded);
        }
    }

    #[test]
    fn viewport_zoom_in_then_out() {
        let mut vp = ViewportState::default();
        let original = vp.orbit.distance;
        vp.zoom(5.0);
        assert!(vp.orbit.distance < original);
        vp.zoom(-5.0);
        assert!(vp.orbit.distance > vp.orbit.min_distance);
    }

    #[test]
    fn viewport_camera_position_updates_on_rotate() {
        let mut vp = ViewportState::default();
        let start_pos = vp.camera.position;
        vp.rotate(100.0, 0.0);
        assert_ne!(vp.camera.position.x, start_pos.x);
    }

    #[test]
    fn viewport_camera_target_stays_zero() {
        let mut vp = ViewportState::default();
        vp.rotate(50.0, 30.0);
        vp.zoom(2.0);
        assert_eq!(vp.camera.target, hisab::Vec3::ZERO);
    }
}
