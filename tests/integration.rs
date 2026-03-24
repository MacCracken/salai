//! Integration tests for salai.

use salai::editor::{EditorApp, EditorState, PlayState};
use salai::expr::{eval_f64, eval_or, eval_or_parse};
use salai::hierarchy::{build_hierarchy, flatten_hierarchy};
use salai::history::{Action, History};
use salai::hw::{HardwareProfile, QualityTier};
use salai::inspector::inspect_entity;
use salai::personality::NpcPersonality;
use salai::viewport::{GizmoMode, ViewportState};

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
    assert_eq!(vp.gizmo_mode, GizmoMode::Translate);
}

#[test]
fn editor_state_full_serde_roundtrip() {
    let entity = kiran::Entity::new(42, 3);
    let mut state = EditorState::default();
    state.play_state = PlayState::Paused;
    state.select(entity);
    state.show_inspector = false;
    state.show_hierarchy = true;
    state.show_viewport = false;
    state.scene_path = Some("levels/arena.toml".into());

    let json = serde_json::to_string(&state).unwrap();
    let decoded: EditorState = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.play_state, PlayState::Paused);
    // Verify generation is preserved through serde round-trip
    let selected = decoded.selected().unwrap();
    assert_eq!(selected.index(), 42);
    assert_eq!(selected.generation(), 3);
    assert!(!decoded.show_inspector);
    assert!(decoded.show_hierarchy);
    assert!(!decoded.show_viewport);
    assert_eq!(decoded.scene_path.as_deref(), Some("levels/arena.toml"));
}

#[test]
fn hierarchy_with_despawned_children() {
    let mut world = kiran::World::new();
    let root = world.spawn();
    let alive = world.spawn();
    let dead = world.spawn();

    world
        .insert_component(root, kiran::scene::Name("Root".into()))
        .unwrap();
    world
        .insert_component(alive, kiran::scene::Name("Alive".into()))
        .unwrap();
    world
        .insert_component(dead, kiran::scene::Name("Dead".into()))
        .unwrap();

    kiran::scene::set_parent(&mut world, alive, root).unwrap();
    kiran::scene::set_parent(&mut world, dead, root).unwrap();
    world.despawn(dead).unwrap();

    let tree = build_hierarchy(&world, &[root, alive, dead]);
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].children.len(), 1);

    let flat = flatten_hierarchy(&tree);
    assert_eq!(flat.len(), 2);
    assert_eq!(flat[1].2, "Alive");
}

#[test]
fn inspector_after_component_removal() {
    let mut world = kiran::World::new();
    let e = world.spawn();
    world
        .insert_component(e, kiran::scene::Name("Test".into()))
        .unwrap();
    world
        .insert_component(e, kiran::scene::Position(hisab::Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();

    assert_eq!(inspect_entity(&world, e).len(), 2);

    world.remove_component::<kiran::scene::Position>(e).unwrap();
    let info = inspect_entity(&world, e);
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].name, "Name");
}

#[test]
fn editor_multi_step_simulation() {
    let mut app = EditorApp::new();
    app.state.play_state = PlayState::Paused;

    for _ in 0..10 {
        app.step_frame();
    }

    let clock = app.world.get_resource::<kiran::GameClock>().unwrap();
    assert_eq!(clock.frame, 10);
}

#[test]
fn full_editor_with_entities_and_viewport() {
    let mut app = EditorApp::new();
    let mut vp = ViewportState::default();

    // Spawn entities
    let e1 = app.world.spawn();
    let e2 = app.world.spawn();
    app.world
        .insert_component(e1, kiran::scene::Name("Player".into()))
        .unwrap();
    app.world
        .insert_component(e1, kiran::scene::Position(hisab::Vec3::new(0.0, 1.0, 0.0)))
        .unwrap();
    app.world
        .insert_component(e2, kiran::scene::Name("Light".into()))
        .unwrap();
    app.world
        .insert_component(e2, kiran::scene::LightComponent { intensity: 1.0 })
        .unwrap();

    // Select and inspect
    app.state.select(e1);
    let selected = app.state.selected().unwrap();
    let info = inspect_entity(&app.world, selected);
    assert_eq!(info.len(), 2);

    // Build hierarchy
    let tree = build_hierarchy(&app.world, &[e1, e2]);
    assert_eq!(tree.len(), 2);

    // Manipulate viewport
    vp.rotate(30.0, 15.0);
    vp.zoom(2.0);
    vp.cycle_gizmo();
    assert_eq!(vp.gizmo_mode, GizmoMode::Rotate);

    // Switch to play mode
    app.state.toggle_play();
    assert!(app.state.is_playing());
}

#[test]
fn expr_as_inspector_input() {
    // Simulate user typing expressions into inspector property fields
    let mut world = kiran::World::new();
    let e = world.spawn();

    // User types "2*pi" into a position X field
    let x = eval_f64("2 * pi").unwrap();
    let y = eval_or("bad input", 0.0);
    let z = eval_or_parse("1.5").unwrap();

    world
        .insert_component(
            e,
            kiran::scene::Position(hisab::Vec3::new(x as f32, y as f32, z as f32)),
        )
        .unwrap();

    let info = inspect_entity(&world, e);
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].name, "Position");
    assert!(info[0].details.contains("6.28")); // 2*pi ≈ 6.28
}

#[test]
fn expr_batch_evaluation() {
    // Evaluate multiple expressions as a user might when filling out a transform
    let exprs = [
        "sin(pi/6)",
        "cos(pi/3)",
        "sqrt(2)/2",
        "1+1",
        "45 * pi / 180",
    ];
    let results: Vec<f64> = exprs.iter().map(|e| eval_f64(e).unwrap()).collect();

    assert_eq!(results.len(), 5);
    for v in &results {
        assert!(v.is_finite());
    }
    // sin(pi/6) ≈ 0.5
    assert!((results[0] - 0.5).abs() < 1e-10);
}

#[test]
fn hw_detect_and_configure_viewport() {
    let profile = HardwareProfile::detect();

    // Profile should always be valid
    assert!(!profile.device_name.is_empty());

    // Quality tier should be valid
    assert!(!format!("{}", profile.quality).is_empty());
}

#[test]
fn hw_quality_tier_values() {
    // Verify all tier variants display correctly
    assert_eq!(QualityTier::Low.to_string(), "Low");
    assert_eq!(QualityTier::Medium.to_string(), "Medium");
    assert_eq!(QualityTier::High.to_string(), "High");
    assert_eq!(QualityTier::Ultra.to_string(), "Ultra");
}

#[test]
fn hw_memory_display() {
    let mut profile = HardwareProfile::default();
    assert_eq!(profile.gpu_memory_display(), "N/A");

    profile.gpu_memory_bytes = 4 * 1024 * 1024 * 1024;
    assert_eq!(profile.gpu_memory_display(), "4.0 GiB");
}

#[test]
fn history_editor_workflow() {
    // Simulate a real editing session with undo/redo
    let mut app = EditorApp::new();
    let mut history = History::new();

    // Spawn an entity and record the action
    let e = app.world.spawn();
    app.world
        .insert_component(e, kiran::scene::Name("Player".into()))
        .unwrap();
    history.record(
        "editor",
        Action::new("spawn_entity", serde_json::json!({"entity_id": e.id()})),
    );

    // Set position and record
    app.world
        .insert_component(e, kiran::scene::Position(hisab::Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();
    history.record(
        "inspector",
        Action::new(
            "set_position",
            serde_json::json!({
                "entity": e.id(),
                "before": [0, 0, 0],
                "after": [1, 2, 3]
            }),
        ),
    );

    assert_eq!(history.len(), 2);
    assert!(history.verify());

    // Undo the position change
    let entry = history.undo().unwrap();
    assert_eq!(entry.action(), "set_position");

    // Undo the spawn
    let entry = history.undo().unwrap();
    assert_eq!(entry.action(), "spawn_entity");

    // Redo both
    history.redo();
    history.redo();
    assert_eq!(history.cursor(), 2);
    assert!(!history.can_redo());
}

#[test]
fn history_integrity_after_many_actions() {
    let mut history = History::new();
    for i in 0..50 {
        history.record(
            "test",
            Action::with_kind(format!("action_{i}"), serde_json::json!({"step": i})),
        );
    }
    assert_eq!(history.len(), 50);
    assert!(history.verify());
}

#[test]
fn personality_editing_workflow() {
    use bhava::traits::{TraitGroup, TraitKind, TraitLevel};

    // Create NPC personalities for a game scene
    let mut guard = NpcPersonality::new("Guard");
    guard.set_trait(TraitKind::Confidence, TraitLevel::High);
    guard.set_trait(TraitKind::Warmth, TraitLevel::Low);
    guard.set_trait(TraitKind::Precision, TraitLevel::High);

    let mut merchant = NpcPersonality::new("Merchant");
    merchant.set_trait(TraitKind::Humor, TraitLevel::High);
    merchant.set_trait(TraitKind::Warmth, TraitLevel::Highest);
    merchant.set_group(TraitGroup::Social, TraitLevel::High);

    // Check inspector summaries
    let guard_summary = guard.inspector_summary();
    assert_eq!(guard_summary.name, "Guard");
    assert_eq!(guard_summary.trait_count, 3);

    let merchant_summary = merchant.inspector_summary();
    assert_eq!(merchant_summary.name, "Merchant");

    // Compatibility check
    let compat = guard.compatibility(&merchant);
    assert!(compat >= 0.0 && compat <= 1.0);

    // Blend for offspring/hybrid NPC
    let hybrid = guard.blend(&merchant, 0.5);
    let hybrid_summary = hybrid.inspector_summary();
    assert!(!hybrid_summary.name.is_empty());
}

#[test]
fn personality_with_mood() {
    let mut npc = NpcPersonality::new("Angry Villager");
    npc.mood.frustration = 0.9;
    npc.mood.joy = -0.5;

    let summary = npc.inspector_summary();
    assert_eq!(summary.mood_label, "Frustrated");
}

#[test]
fn texture_inspect_and_thumbnail() {
    // Create a 4x4 red texture
    let mut data = Vec::with_capacity(4 * 4 * 4);
    for _ in 0..16 {
        data.extend_from_slice(&[255, 0, 0, 255]);
    }

    let info = salai::texture::inspect_texture(&data, 4, 4);
    assert_eq!(info.width, 4);
    assert_eq!(info.height, 4);
    assert_eq!(info.average_color, [255, 0, 0, 255]);

    // Hex display
    let hex = salai::texture::color_to_hex(info.average_color);
    assert_eq!(hex, "#FF0000FF");
}

#[test]
fn audio_inspect_and_waveform() {
    // Create a 0.1 second sine wave
    let samples: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
        .collect();
    let buf = salai::audio::buffer_from_samples(samples, 1, 44100).unwrap();

    let info = salai::audio::inspect_audio(&buf);
    assert!((info.duration_secs - 0.1).abs() < 0.01);
    assert_eq!(info.channels, 1);
    assert!(info.peak > 0.9);

    // Waveform
    let wf = salai::audio::waveform(&buf, 100);
    assert!(!wf.channels.is_empty());

    // Duration formatting
    assert_eq!(
        salai::audio::format_duration(info.duration_secs),
        "0:00.100"
    );
}
