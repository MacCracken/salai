//! Viewport panel — 3D scene rendering with orbit camera and grid.
//!
//! Renders the scene using soorat's GPU pipeline into an offscreen texture,
//! then displays it in an egui panel. Handles mouse interaction for camera
//! orbit, zoom, and entity selection.

use crate::editor::EditorState;
use crate::viewport::{GizmoMode, ViewportState};

/// Render viewport with camera controls, scene display, and entity picking.
pub fn viewport_panel_with_picking(
    ui: &mut egui::Ui,
    viewport: &mut ViewportState,
    picking: Option<(&kiran::World, &[kiran::Entity], &mut EditorState)>,
) {
    let available = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());

    // Handle mouse interaction
    handle_mouse_input(ui, &response, viewport);

    // Handle entity picking on click
    if response.clicked() {
        if let Some((world, entities, state)) = picking {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let (ndc_x, ndc_y) = crate::picking::pixel_to_ndc(
                    pointer_pos.x - rect.left(),
                    pointer_pos.y - rect.top(),
                    rect.width(),
                    rect.height(),
                );
                let view_proj = viewport.camera.view_projection();
                if let Some(hit) = crate::picking::pick_entity(
                    world,
                    entities,
                    viewport.camera.position,
                    view_proj,
                    ndc_x,
                    ndc_y,
                    0.5,
                ) {
                    let modifiers = ui.input(|i| i.modifiers);
                    if modifiers.shift {
                        state.select_add(hit.entity);
                    } else if modifiers.ctrl || modifiers.mac_cmd {
                        state.select_toggle(hit.entity);
                    } else {
                        state.select(hit.entity);
                    }
                    tracing::info!(
                        entity = %hit.entity,
                        distance = hit.distance,
                        "entity picked in viewport"
                    );
                }
            }
        }
    }

    // Draw viewport background
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 35));

    // Draw grid
    if viewport.show_grid {
        draw_grid(&painter, rect, viewport);
    }

    // Draw scene entities placeholder
    draw_scene_placeholder(&painter, rect, viewport);

    // Draw gizmo indicator
    draw_gizmo_indicator(&painter, rect, viewport.gizmo_mode);

    // Draw viewport info overlay
    draw_info_overlay(&painter, rect, viewport);
}

/// Handle mouse drag for orbit, scroll for zoom.
fn handle_mouse_input(ui: &mut egui::Ui, response: &egui::Response, viewport: &mut ViewportState) {
    // Left-drag: orbit camera
    if response.dragged_by(egui::PointerButton::Primary) {
        let delta = response.drag_delta();
        viewport.rotate(delta.x, delta.y);
    }

    // Scroll: zoom
    let scroll = ui.input(|i| i.raw_scroll_delta.y);
    if scroll.abs() > 0.1 {
        viewport.zoom(scroll * 0.1);
    }

    // Right-click: context menu (future)
    // Middle-drag: pan (future V0.4)
}

/// Draw a ground-plane grid projected into the viewport.
fn draw_grid(painter: &egui::Painter, rect: egui::Rect, viewport: &ViewportState) {
    let center = rect.center();
    let grid_color = egui::Color32::from_rgba_premultiplied(80, 80, 80, 60);
    let axis_color = egui::Color32::from_rgba_premultiplied(120, 120, 120, 100);

    let grid_count = 10;
    let cell_px = (rect.width().min(rect.height()) / (grid_count as f32 * 2.0))
        * (5.0 / viewport.orbit.distance).clamp(0.3, 3.0);

    // Grid lines
    for i in -grid_count..=grid_count {
        let offset = i as f32 * cell_px;
        let color = if i == 0 { axis_color } else { grid_color };

        // Horizontal
        painter.line_segment(
            [
                egui::pos2(rect.left(), center.y + offset),
                egui::pos2(rect.right(), center.y + offset),
            ],
            egui::Stroke::new(if i == 0 { 1.5 } else { 0.5 }, color),
        );

        // Vertical
        painter.line_segment(
            [
                egui::pos2(center.x + offset, rect.top()),
                egui::pos2(center.x + offset, rect.bottom()),
            ],
            egui::Stroke::new(if i == 0 { 1.5 } else { 0.5 }, color),
        );
    }
}

/// Draw a placeholder for scene entities (replaced by soorat 3D rendering later).
fn draw_scene_placeholder(painter: &egui::Painter, rect: egui::Rect, viewport: &ViewportState) {
    let center = rect.center();

    // Origin marker
    let origin_size = 4.0;
    painter.circle_filled(center, origin_size, egui::Color32::from_rgb(200, 200, 200));

    // Camera direction indicator
    let yaw = viewport.orbit.yaw;
    let arrow_len = 30.0;
    painter.arrow(
        center,
        egui::vec2(yaw.sin() * arrow_len, -yaw.cos() * arrow_len),
        egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 150, 255)),
    );
}

/// Draw the active gizmo mode indicator in the corner.
fn draw_gizmo_indicator(painter: &egui::Painter, rect: egui::Rect, mode: GizmoMode) {
    let (label, color) = match mode {
        GizmoMode::Translate => ("Move", egui::Color32::from_rgb(80, 180, 80)),
        GizmoMode::Rotate => ("Rotate", egui::Color32::from_rgb(80, 120, 220)),
        GizmoMode::Scale => ("Scale", egui::Color32::from_rgb(220, 160, 60)),
    };

    let pos = egui::pos2(rect.left() + 10.0, rect.top() + 10.0);
    painter.text(
        pos,
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(12.0),
        color,
    );
}

/// Draw camera info overlay in the bottom-right.
fn draw_info_overlay(painter: &egui::Painter, rect: egui::Rect, viewport: &ViewportState) {
    let text = format!(
        "dist: {:.1}  yaw: {:.0}  pitch: {:.0}",
        viewport.orbit.distance,
        viewport.orbit.yaw.to_degrees(),
        viewport.orbit.pitch.to_degrees(),
    );

    let pos = egui::pos2(rect.right() - 10.0, rect.bottom() - 10.0);
    painter.text(
        pos,
        egui::Align2::RIGHT_BOTTOM,
        text,
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(150, 150, 150),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_state_defaults() {
        let vp = ViewportState::default();
        assert!(vp.show_grid);
        assert_eq!(vp.gizmo_mode, GizmoMode::Translate);
    }

    #[test]
    fn rotate_updates_camera() {
        let mut vp = ViewportState::default();
        let start_yaw = vp.orbit.yaw;
        vp.rotate(50.0, 0.0);
        assert_ne!(vp.orbit.yaw, start_yaw);
        assert!(vp.camera.position.x.is_finite());
    }

    #[test]
    fn zoom_clamps() {
        let mut vp = ViewportState::default();
        vp.zoom(10000.0);
        assert_eq!(vp.orbit.distance, vp.orbit.min_distance);
        vp.zoom(-10000.0);
        assert_eq!(vp.orbit.distance, vp.orbit.max_distance);
    }

    #[test]
    fn grid_toggle() {
        let mut vp = ViewportState::default();
        vp.show_grid = false;
        assert!(!vp.show_grid);
    }
}
