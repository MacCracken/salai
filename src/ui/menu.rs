//! Editor menu bar — File, Edit, View menus.

use crate::editor::EditorApp;
use muharrir::History;

/// Extra panel visibility flags not stored in EditorState.
pub struct PanelFlags<'a> {
    pub show_console: &'a mut bool,
    pub show_profiler: &'a mut bool,
    pub show_assets: &'a mut bool,
}

/// Render the menu bar.
pub fn menu_bar(
    ui: &mut egui::Ui,
    editor: &mut EditorApp,
    history: &mut History,
    panels: PanelFlags<'_>,
) {
    let state = &mut editor.state;
    egui::menu::bar(ui, |ui| {
        // File menu
        ui.menu_button("File", |ui| {
            if ui.button("Open Scene...").clicked() {
                // TODO: file dialog integration
                tracing::info!("open scene requested");
                ui.close_menu();
            }
            if ui.button("Save Scene").clicked() {
                if let Some(path) = &state.scene_path {
                    let scene = crate::scene_edit::extract_scene(
                        &editor.world,
                        &editor.tracked_entities,
                        &path.clone(),
                    );
                    if let Err(e) = crate::scene_edit::save_scene(&scene, path) {
                        tracing::error!(error = %e, "failed to save scene");
                    }
                } else {
                    tracing::warn!("no scene path set — use Save As");
                }
                ui.close_menu();
            }
            if ui.button("Save Scene As...").clicked() {
                tracing::info!("save scene as requested");
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Quit").clicked() {
                tracing::info!("quit requested");
                ui.close_menu();
            }
        });

        // Edit menu
        ui.menu_button("Edit", |ui| {
            if ui
                .add_enabled(history.can_undo(), egui::Button::new("Undo"))
                .clicked()
            {
                if let Some(entry) = history.undo() {
                    tracing::info!(action = entry.action(), "undo");
                }
                ui.close_menu();
            }
            if ui
                .add_enabled(history.can_redo(), egui::Button::new("Redo"))
                .clicked()
            {
                if let Some(entry) = history.redo() {
                    tracing::info!(action = entry.action(), "redo");
                }
                ui.close_menu();
            }
        });

        // View menu
        ui.menu_button("View", |ui| {
            if ui
                .checkbox(&mut state.show_inspector, "Inspector")
                .changed()
            {
                tracing::debug!(show = state.show_inspector, "inspector toggled");
            }
            if ui
                .checkbox(&mut state.show_hierarchy, "Hierarchy")
                .changed()
            {
                tracing::debug!(show = state.show_hierarchy, "hierarchy toggled");
            }
            if ui.checkbox(&mut state.show_viewport, "Viewport").changed() {
                tracing::debug!(show = state.show_viewport, "viewport toggled");
            }
            ui.separator();
            ui.checkbox(panels.show_console, "Console");
            ui.checkbox(panels.show_profiler, "Profiler");
            ui.checkbox(panels.show_assets, "Assets");
        });
    });
}

#[cfg(test)]
mod tests {
    use crate::editor::EditorState;
    use muharrir::History;

    #[test]
    fn menu_state_defaults() {
        let state = EditorState::default();
        assert!(state.show_inspector);
        assert!(state.show_hierarchy);
        assert!(state.show_viewport);
    }

    #[test]
    fn history_undo_redo_state() {
        let history = History::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
