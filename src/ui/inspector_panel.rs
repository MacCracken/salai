//! Inspector panel — view and edit components on the selected entity.

use crate::editor::EditorState;
use crate::inspector::inspect_entity;

/// Render the inspector panel for the currently selected entity.
pub fn inspector_panel(ui: &mut egui::Ui, world: &kiran::World, state: &EditorState) {
    ui.heading("Inspector");
    ui.separator();

    let Some(entity) = state.selected() else {
        ui.label("No entity selected");
        return;
    };

    if !world.is_alive(entity) {
        ui.colored_label(egui::Color32::RED, "Entity no longer alive");
        return;
    }

    ui.label(format!("Entity: {entity}"));
    ui.separator();

    let components = inspect_entity(world, entity);

    if components.is_empty() {
        ui.label("No components");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for comp in &components {
            egui::CollapsingHeader::new(comp.name)
                .default_open(true)
                .show(ui, |ui| {
                    // For now, display as read-only text
                    // V0.4 will add editable fields with expr evaluation
                    ui.label(&comp.details);
                });
        }
    });
}

/// Render an expression-evaluable numeric field.
/// Returns the evaluated value if the input changed.
#[allow(dead_code)]
pub fn expr_field(ui: &mut egui::Ui, label: &str, value: &mut f64, buf: &mut String) -> bool {
    ui.horizontal(|ui| {
        ui.label(label);
        let response = ui.text_edit_singleline(buf);
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(v) = muharrir::eval_f64(buf) {
                *value = v;
                *buf = format!("{v:.4}");
                tracing::debug!(label, value = v, "expression evaluated");
                return true;
            }
        }
        false
    })
    .inner
}

#[cfg(test)]
mod tests {
    use crate::editor::EditorState;
    use crate::inspector::inspect_entity;

    #[test]
    fn inspector_no_selection() {
        let state = EditorState::default();
        assert!(state.selected().is_none());
    }

    #[test]
    fn inspector_dead_entity() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world
            .insert_component(e, kiran::scene::Name("Test".into()))
            .unwrap();
        world.despawn(e).unwrap();
        assert!(!world.is_alive(e));
        assert!(inspect_entity(&world, e).is_empty());
    }

    #[test]
    fn inspector_with_components() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world
            .insert_component(e, kiran::scene::Name("Player".into()))
            .unwrap();
        world
            .insert_component(e, kiran::scene::Position(hisab::Vec3::new(1.0, 2.0, 3.0)))
            .unwrap();
        let info = inspect_entity(&world, e);
        assert_eq!(info.len(), 2);
    }

    #[test]
    fn expr_evaluation_works() {
        let result = muharrir::eval_f64("2 * pi").unwrap();
        assert!((result - std::f64::consts::TAU).abs() < 1e-10);
    }
}
