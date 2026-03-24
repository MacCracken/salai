//! Editor UI — egui panels, toolbar, and menu bar.
//!
//! The UI module renders the editor interface using egui/eframe. It reads from
//! and writes to [`EditorApp`] state, and delegates to submodules for each panel.

mod hierarchy_panel;
mod inspector_panel;
mod menu;
mod toolbar;
mod viewport_panel;

use crate::editor::EditorApp;

/// The eframe application wrapper that drives the editor UI.
pub struct SalaiApp {
    pub editor: EditorApp,
    pub viewport: crate::viewport::ViewportState,
    pub history: muharrir::History,
}

impl SalaiApp {
    /// Create a new editor application with optional scene path.
    #[must_use]
    pub fn new(scene_path: Option<&str>) -> Self {
        let mut editor = EditorApp::new();
        if let Some(path) = scene_path {
            if let Err(e) = editor.load_scene(path) {
                tracing::error!(path, error = %e, "failed to load scene");
            }
        }

        Self {
            editor,
            viewport: crate::viewport::ViewportState::default(),
            history: muharrir::History::new(),
        }
    }

    /// Get the tracked entity list.
    #[must_use]
    pub fn entities(&self) -> &[kiran::Entity] {
        self.editor.entities()
    }
}

impl eframe::App for SalaiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            menu::menu_bar(ui, &mut self.editor, &mut self.history);
        });

        // Toolbar below menu
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar::toolbar(ui, &mut self.editor, &mut self.viewport, &mut self.history);
        });

        // Hierarchy panel (left)
        if self.editor.state.show_hierarchy {
            egui::SidePanel::left("hierarchy")
                .default_width(200.0)
                .show(ctx, |ui| {
                    let entities = &self.editor.tracked_entities;
                    hierarchy_panel::hierarchy_panel(
                        ui,
                        &self.editor.world,
                        entities,
                        &mut self.editor.state,
                    );
                });
        }

        // Inspector panel (right)
        if self.editor.state.show_inspector {
            egui::SidePanel::right("inspector")
                .default_width(280.0)
                .show(ctx, |ui| {
                    inspector_panel::inspector_panel(ui, &self.editor.world, &self.editor.state);
                });
        }

        // Central viewport area
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.editor.state.show_viewport {
                viewport_panel::viewport_panel(ui, &mut self.viewport);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Viewport hidden");
                });
            }
        });

        // Step simulation if playing
        if self.editor.state.is_playing() {
            let clock = self
                .editor
                .world
                .get_resource_mut::<kiran::world::GameClock>()
                .unwrap();
            clock.tick(1.0 / 60.0);
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::PlayState;

    #[test]
    fn salai_app_new_empty() {
        let app = SalaiApp::new(None);
        assert_eq!(app.editor.entity_count(), 0);
        assert_eq!(app.editor.state.play_state, PlayState::Editing);
        assert!(app.history.is_empty());
    }

    #[test]
    fn salai_app_new_invalid_scene() {
        let app = SalaiApp::new(Some("/nonexistent.toml"));
        assert_eq!(app.editor.entity_count(), 0);
    }

    #[test]
    fn all_entities_empty_world() {
        let app = SalaiApp::new(None);
        assert!(app.entities().is_empty());
    }

    #[test]
    fn salai_app_with_entities() {
        let mut app = SalaiApp::new(None);
        let e1 = app.editor.spawn_entity();
        let _e2 = app.editor.spawn_entity();
        assert_eq!(app.entities().len(), 2);

        // Select and verify
        app.editor.state.select(e1);
        assert_eq!(app.editor.state.selected(), Some(e1));

        // Despawn clears selection
        app.editor.despawn_entity(e1).unwrap();
        assert_eq!(app.entities().len(), 1);
        assert!(app.editor.state.selected().is_none());
    }

    #[test]
    fn salai_app_viewport_state() {
        let app = SalaiApp::new(None);
        assert!(app.viewport.show_grid);
        assert_eq!(
            app.viewport.gizmo_mode,
            crate::viewport::GizmoMode::Translate
        );
    }
}
