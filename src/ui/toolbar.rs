//! Editor toolbar — play/pause/step controls and gizmo mode selector.

use crate::editor::{EditorApp, PlayState};
use crate::viewport::{GizmoMode, ViewportState};

/// Render the editor toolbar.
pub fn toolbar(
    ui: &mut egui::Ui,
    editor: &mut EditorApp,
    viewport: &mut ViewportState,
    history: &mut muharrir::History,
) {
    ui.horizontal(|ui| {
        // Play/Pause/Stop controls
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let play_label = match editor.state.play_state {
                    PlayState::Editing => "Play",
                    PlayState::Playing => "Pause",
                    PlayState::Paused => "Resume",
                };

                if ui.button(play_label).clicked() {
                    editor.state.toggle_play();
                    tracing::info!(state = ?editor.state.play_state, "play state toggled");
                }

                let can_stop = editor.state.play_state != PlayState::Editing;
                if ui
                    .add_enabled(can_stop, egui::Button::new("Stop"))
                    .clicked()
                {
                    editor.state.stop();
                    tracing::info!("stopped playback");
                }

                let can_step = editor.state.play_state == PlayState::Paused;
                if ui
                    .add_enabled(can_step, egui::Button::new("Step"))
                    .clicked()
                {
                    editor.step_frame();
                    tracing::debug!("stepped one frame");
                }
            });
        });

        ui.separator();

        // Add entity button
        ui.group(|ui| {
            if ui.button("+ Entity").clicked() {
                let name = format!("Entity {}", editor.entity_count() + 1);
                let _entity = crate::scene_edit::add_entity(
                    &mut editor.world,
                    &mut editor.tracked_entities,
                    history,
                    &name,
                );
                tracing::info!(name, "entity added from toolbar");
            }
        });

        ui.separator();

        // Gizmo mode selector
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Gizmo:");
                let modes = [
                    (GizmoMode::Translate, "Move"),
                    (GizmoMode::Rotate, "Rotate"),
                    (GizmoMode::Scale, "Scale"),
                ];
                for (mode, label) in modes {
                    if ui
                        .selectable_label(viewport.gizmo_mode == mode, label)
                        .clicked()
                    {
                        viewport.gizmo_mode = mode;
                        tracing::debug!(?mode, "gizmo mode changed");
                    }
                }
            });
        });

        ui.separator();

        // Status
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("Entities: {}", editor.entity_count()));
            match editor.state.play_state {
                PlayState::Editing => ui.label("Editing"),
                PlayState::Playing => ui.label("Playing"),
                PlayState::Paused => ui.label("Paused"),
            };
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_play_label_editing() {
        let editor = EditorApp::new();
        let label = match editor.state.play_state {
            PlayState::Editing => "Play",
            PlayState::Playing => "Pause",
            PlayState::Paused => "Resume",
        };
        assert_eq!(label, "Play");
    }

    #[test]
    fn toolbar_play_label_playing() {
        let mut editor = EditorApp::new();
        editor.state.toggle_play();
        let label = match editor.state.play_state {
            PlayState::Editing => "Play",
            PlayState::Playing => "Pause",
            PlayState::Paused => "Resume",
        };
        assert_eq!(label, "Pause");
    }

    #[test]
    fn toolbar_gizmo_modes() {
        let modes = [
            (GizmoMode::Translate, "Move"),
            (GizmoMode::Rotate, "Rotate"),
            (GizmoMode::Scale, "Scale"),
        ];
        assert_eq!(modes.len(), 3);
    }
}
