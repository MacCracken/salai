//! Integration tests for salai.

use salai::editor::{EditorApp, PlayState};
use salai::hierarchy::{build_hierarchy, flatten_hierarchy};
use salai::inspector::inspect_entity;
use salai::viewport::ViewportState;

#[test]
fn editor_lifecycle() {
    let mut app = EditorApp::new();
    assert_eq!(app.entity_count(), 0);
    assert_eq!(app.state.play_state, PlayState::Editing);

    // Toggle through states
    app.state.toggle_play();
    assert!(app.state.is_playing());

    app.state.toggle_play();
    assert_eq!(app.state.play_state, PlayState::Paused);

    // Step one frame
    app.step_frame();
    let clock = app.world.get_resource::<kiran::GameClock>().unwrap();
    assert_eq!(clock.frame, 1);

    app.state.stop();
    assert_eq!(app.state.play_state, PlayState::Editing);
}

#[test]
fn inspector_and_hierarchy_integration() {
    let mut world = kiran::World::new();

    // Build a small scene
    let parent = world.spawn();
    let child = world.spawn();
    world
        .insert_component(parent, kiran::scene::Name("Root".into()))
        .unwrap();
    world
        .insert_component(
            parent,
            kiran::scene::Position(hisab::Vec3::new(1.0, 2.0, 3.0)),
        )
        .unwrap();
    world
        .insert_component(child, kiran::scene::Name("Child".into()))
        .unwrap();
    kiran::scene::set_parent(&mut world, child, parent).unwrap();

    // Inspector
    let info = inspect_entity(&world, parent);
    assert_eq!(info.len(), 2); // Name + Position

    // Hierarchy
    let tree = build_hierarchy(&world, &[parent, child]);
    assert_eq!(tree.len(), 1); // only root
    assert_eq!(tree[0].children.len(), 1);

    let flat = flatten_hierarchy(&tree);
    assert_eq!(flat.len(), 2);
    assert_eq!(flat[0].2, "Root");
    assert_eq!(flat[1].2, "Child");
}

#[test]
fn viewport_full_workflow() {
    let mut vp = ViewportState::default();

    // Rotate
    vp.rotate(50.0, 30.0);
    assert_ne!(vp.orbit.yaw, 0.0);

    // Zoom
    let d = vp.orbit.distance;
    vp.zoom(3.0);
    assert!(vp.orbit.distance < d);

    // Cycle gizmos
    vp.cycle_gizmo();
    vp.cycle_gizmo();
    vp.cycle_gizmo();
    assert_eq!(vp.gizmo_mode, salai::viewport::GizmoMode::Translate);
}
