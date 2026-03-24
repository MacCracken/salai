//! Hierarchy panel — collapsible entity tree view.

use crate::editor::EditorState;
use crate::hierarchy::{HierarchyNode, build_hierarchy};

/// Render the hierarchy panel showing the entity tree.
pub fn hierarchy_panel(
    ui: &mut egui::Ui,
    world: &kiran::World,
    entities: &[kiran::Entity],
    state: &mut EditorState,
) {
    ui.heading("Hierarchy");
    ui.separator();

    let tree = build_hierarchy(world, entities);

    if tree.is_empty() {
        ui.label("No entities in scene");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for node in &tree {
            render_node(ui, node, state);
        }
    });
}

/// Recursively render a hierarchy node with collapsible children.
fn render_node(ui: &mut egui::Ui, node: &HierarchyNode, state: &mut EditorState) {
    let selected = state.is_selected(node.entity);
    let has_children = !node.children.is_empty();

    let handle_click = |ui: &egui::Ui, state: &mut EditorState| {
        let modifiers = ui.input(|i| i.modifiers);
        if modifiers.shift {
            state.select_add(node.entity);
        } else if modifiers.ctrl || modifiers.mac_cmd {
            state.select_toggle(node.entity);
        } else {
            state.select(node.entity);
        }
        tracing::debug!(
            entity = %node.entity,
            name = %node.name,
            count = state.selection_count(),
            "entity selected"
        );
    };

    if has_children {
        let id = ui.make_persistent_id(node.entity.id());
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                if ui.selectable_label(selected, &node.name).clicked() {
                    handle_click(ui, state);
                }
            })
            .body(|ui| {
                for child in &node.children {
                    render_node(ui, child, state);
                }
            });
    } else if ui.selectable_label(selected, &node.name).clicked() {
        handle_click(ui, state);
    }
}

#[cfg(test)]
mod tests {
    use crate::editor::EditorState;
    use crate::hierarchy::{HierarchyNode, build_hierarchy};

    #[test]
    fn empty_world_produces_no_nodes() {
        let world = kiran::World::new();
        let tree = build_hierarchy(&world, &[]);
        assert!(tree.is_empty());
    }

    #[test]
    fn selection_tracks_entity() {
        let mut state = EditorState::default();
        let entity = kiran::Entity::new(1, 0);
        state.select(entity);
        assert_eq!(state.selected(), Some(entity));
    }

    #[test]
    fn hierarchy_node_with_children() {
        let node = HierarchyNode {
            entity: kiran::Entity::new(1, 0),
            name: "Root".into(),
            children: vec![HierarchyNode {
                entity: kiran::Entity::new(2, 0),
                name: "Child".into(),
                children: vec![],
                depth: 1,
            }],
            depth: 0,
        };
        assert!(!node.children.is_empty());
        assert_eq!(node.children[0].name, "Child");
    }
}
