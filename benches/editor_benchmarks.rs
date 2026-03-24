use criterion::{Criterion, black_box, criterion_group, criterion_main};

use salai::editor::{EditorApp, EditorState, PlayState};
use salai::expr::{eval_f64, eval_or, eval_or_parse};
use salai::hierarchy::{build_hierarchy, flatten_hierarchy};
use salai::history::{Action, History};
use salai::hw::HardwareProfile;
use salai::inspector::inspect_entity;
use salai::personality::NpcPersonality;
use salai::viewport::ViewportState;

// ---------------------------------------------------------------------------
// Editor benchmarks
// ---------------------------------------------------------------------------

fn bench_editor_app_new(c: &mut Criterion) {
    c.bench_function("editor_app_new", |b| {
        b.iter(|| black_box(EditorApp::new()));
    });
}

fn bench_editor_state_toggle_play(c: &mut Criterion) {
    c.bench_function("editor_state_toggle_play", |b| {
        let mut state = EditorState::default();
        b.iter(|| {
            state.toggle_play();
            black_box(&state);
        });
    });
}

fn bench_editor_step_frame(c: &mut Criterion) {
    c.bench_function("editor_step_frame", |b| {
        let mut app = EditorApp::new();
        app.state.play_state = PlayState::Paused;
        b.iter(|| {
            app.step_frame();
            black_box(&app.state);
        });
    });
}

fn bench_editor_state_serde(c: &mut Criterion) {
    c.bench_function("editor_state_serde_roundtrip", |b| {
        let state = EditorState::default();
        b.iter(|| {
            let json = serde_json::to_string(&state).unwrap();
            let decoded: EditorState = serde_json::from_str(&json).unwrap();
            black_box(decoded);
        });
    });
}

// ---------------------------------------------------------------------------
// Inspector benchmarks
// ---------------------------------------------------------------------------

fn bench_inspect_empty_entity(c: &mut Criterion) {
    let mut world = kiran::World::new();
    let e = world.spawn();

    c.bench_function("inspect_empty_entity", |b| {
        b.iter(|| black_box(inspect_entity(&world, e)));
    });
}

fn bench_inspect_entity_full(c: &mut Criterion) {
    let mut world = kiran::World::new();
    let e = world.spawn();
    world
        .insert_component(e, kiran::scene::Name("Bench".into()))
        .unwrap();
    world
        .insert_component(e, kiran::scene::Position(hisab::Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();
    world
        .insert_component(e, kiran::scene::LightComponent { intensity: 0.8 })
        .unwrap();
    world
        .insert_component(
            e,
            kiran::scene::Tags(vec!["a".into(), "b".into(), "c".into()]),
        )
        .unwrap();
    world
        .insert_component(
            e,
            kiran::scene::Material {
                color: [1.0, 0.0, 0.0, 1.0],
                texture: Some("tex.png".into()),
            },
        )
        .unwrap();

    c.bench_function("inspect_entity_5_components", |b| {
        b.iter(|| black_box(inspect_entity(&world, e)));
    });
}

// ---------------------------------------------------------------------------
// Hierarchy benchmarks
// ---------------------------------------------------------------------------

fn bench_hierarchy_flat_10(c: &mut Criterion) {
    let mut world = kiran::World::new();
    let entities: Vec<_> = (0..10)
        .map(|i| {
            let e = world.spawn();
            world
                .insert_component(e, kiran::scene::Name(format!("E{i}")))
                .unwrap();
            e
        })
        .collect();

    c.bench_function("hierarchy_flat_10", |b| {
        b.iter(|| black_box(build_hierarchy(&world, &entities)));
    });
}

fn bench_hierarchy_flat_100(c: &mut Criterion) {
    let mut world = kiran::World::new();
    let entities: Vec<_> = (0..100)
        .map(|i| {
            let e = world.spawn();
            world
                .insert_component(e, kiran::scene::Name(format!("E{i}")))
                .unwrap();
            e
        })
        .collect();

    c.bench_function("hierarchy_flat_100", |b| {
        b.iter(|| black_box(build_hierarchy(&world, &entities)));
    });
}

fn bench_hierarchy_deep_chain(c: &mut Criterion) {
    let mut world = kiran::World::new();
    let mut entities = Vec::new();
    let root = world.spawn();
    world
        .insert_component(root, kiran::scene::Name("Root".into()))
        .unwrap();
    entities.push(root);

    let mut parent = root;
    for i in 1..20 {
        let child = world.spawn();
        world
            .insert_component(child, kiran::scene::Name(format!("L{i}")))
            .unwrap();
        kiran::scene::set_parent(&mut world, child, parent).unwrap();
        entities.push(child);
        parent = child;
    }

    c.bench_function("hierarchy_deep_chain_20", |b| {
        b.iter(|| black_box(build_hierarchy(&world, &entities)));
    });
}

fn bench_hierarchy_wide(c: &mut Criterion) {
    let mut world = kiran::World::new();
    let root = world.spawn();
    world
        .insert_component(root, kiran::scene::Name("Root".into()))
        .unwrap();
    let mut entities = vec![root];

    for i in 0..50 {
        let child = world.spawn();
        world
            .insert_component(child, kiran::scene::Name(format!("C{i}")))
            .unwrap();
        kiran::scene::set_parent(&mut world, child, root).unwrap();
        entities.push(child);
    }

    c.bench_function("hierarchy_wide_50_children", |b| {
        b.iter(|| black_box(build_hierarchy(&world, &entities)));
    });
}

fn bench_flatten_hierarchy(c: &mut Criterion) {
    let mut world = kiran::World::new();
    let root = world.spawn();
    world
        .insert_component(root, kiran::scene::Name("Root".into()))
        .unwrap();
    let mut entities = vec![root];

    for i in 0..50 {
        let child = world.spawn();
        world
            .insert_component(child, kiran::scene::Name(format!("C{i}")))
            .unwrap();
        kiran::scene::set_parent(&mut world, child, root).unwrap();
        entities.push(child);
    }

    let tree = build_hierarchy(&world, &entities);

    c.bench_function("flatten_hierarchy_51_nodes", |b| {
        b.iter(|| black_box(flatten_hierarchy(&tree)));
    });
}

// ---------------------------------------------------------------------------
// Viewport benchmarks
// ---------------------------------------------------------------------------

fn bench_viewport_default(c: &mut Criterion) {
    c.bench_function("viewport_default", |b| {
        b.iter(|| black_box(ViewportState::default()));
    });
}

fn bench_viewport_rotate(c: &mut Criterion) {
    let mut vp = ViewportState::default();
    c.bench_function("viewport_rotate", |b| {
        b.iter(|| {
            vp.rotate(1.0, 0.5);
            black_box(&vp);
        });
    });
}

fn bench_viewport_zoom(c: &mut Criterion) {
    let mut vp = ViewportState::default();
    c.bench_function("viewport_zoom", |b| {
        b.iter(|| {
            vp.zoom(0.1);
            black_box(&vp);
        });
    });
}

fn bench_viewport_cycle_gizmo(c: &mut Criterion) {
    let mut vp = ViewportState::default();
    c.bench_function("viewport_cycle_gizmo", |b| {
        b.iter(|| {
            vp.cycle_gizmo();
            black_box(&vp);
        });
    });
}

// ---------------------------------------------------------------------------
// Expression benchmarks
// ---------------------------------------------------------------------------

fn bench_eval_simple_arithmetic(c: &mut Criterion) {
    c.bench_function("eval_simple_arithmetic", |b| {
        b.iter(|| black_box(eval_f64("1 + 2 * 3")));
    });
}

fn bench_eval_complex_expression(c: &mut Criterion) {
    c.bench_function("eval_complex_expression", |b| {
        b.iter(|| black_box(eval_f64("sqrt(sin(pi/4)^2 + cos(pi/4)^2)")));
    });
}

fn bench_eval_plain_number(c: &mut Criterion) {
    c.bench_function("eval_plain_number", |b| {
        b.iter(|| black_box(eval_f64("42.5")));
    });
}

fn bench_eval_or_fallback(c: &mut Criterion) {
    c.bench_function("eval_or_fallback", |b| {
        b.iter(|| black_box(eval_or("bad_expr", 0.0)));
    });
}

fn bench_eval_or_parse_number(c: &mut Criterion) {
    c.bench_function("eval_or_parse_number", |b| {
        b.iter(|| black_box(eval_or_parse("3.14")));
    });
}

// ---------------------------------------------------------------------------
// Hardware detection benchmarks
// ---------------------------------------------------------------------------

fn bench_hw_detect(c: &mut Criterion) {
    c.bench_function("hw_detect", |b| {
        b.iter(|| black_box(HardwareProfile::detect()));
    });
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

criterion_group!(
    editor_benches,
    bench_editor_app_new,
    bench_editor_state_toggle_play,
    bench_editor_step_frame,
    bench_editor_state_serde,
);

criterion_group!(
    inspector_benches,
    bench_inspect_empty_entity,
    bench_inspect_entity_full,
);

criterion_group!(
    hierarchy_benches,
    bench_hierarchy_flat_10,
    bench_hierarchy_flat_100,
    bench_hierarchy_deep_chain,
    bench_hierarchy_wide,
    bench_flatten_hierarchy,
);

criterion_group!(
    viewport_benches,
    bench_viewport_default,
    bench_viewport_rotate,
    bench_viewport_zoom,
    bench_viewport_cycle_gizmo,
);

criterion_group!(
    expr_benches,
    bench_eval_simple_arithmetic,
    bench_eval_complex_expression,
    bench_eval_plain_number,
    bench_eval_or_fallback,
    bench_eval_or_parse_number,
);

criterion_group!(hw_benches, bench_hw_detect,);

// ---------------------------------------------------------------------------
// History benchmarks
// ---------------------------------------------------------------------------

fn bench_history_record(c: &mut Criterion) {
    c.bench_function("history_record", |b| {
        let mut h = History::new();
        let details = serde_json::json!({"entity": 1, "field": "position", "before": [0,0,0], "after": [1,2,3]});
        b.iter(|| {
            h.record("bench", Action::new("set_position", details.clone()));
            black_box(&h);
        });
    });
}

fn bench_history_undo_redo(c: &mut Criterion) {
    let mut h = History::new();
    for i in 0..100 {
        h.record("bench", Action::new("action", serde_json::json!({"i": i})));
    }

    c.bench_function("history_undo_redo_cycle", |b| {
        b.iter(|| {
            h.undo();
            h.redo();
            black_box(&h);
        });
    });
}

fn bench_history_verify_100(c: &mut Criterion) {
    let mut h = History::new();
    for i in 0..100 {
        h.record("bench", Action::new("action", serde_json::json!({"i": i})));
    }

    c.bench_function("history_verify_100_entries", |b| {
        b.iter(|| black_box(h.verify()));
    });
}

criterion_group!(
    history_benches,
    bench_history_record,
    bench_history_undo_redo,
    bench_history_verify_100,
);

// ---------------------------------------------------------------------------
// Personality benchmarks
// ---------------------------------------------------------------------------

fn bench_npc_personality_new(c: &mut Criterion) {
    c.bench_function("npc_personality_new", |b| {
        b.iter(|| black_box(NpcPersonality::new("Guard")));
    });
}

fn bench_npc_personality_inspector_summary(c: &mut Criterion) {
    let mut npc = NpcPersonality::new("Boss");
    npc.set_trait(
        bhava::traits::TraitKind::Confidence,
        bhava::traits::TraitLevel::Highest,
    );
    npc.set_trait(
        bhava::traits::TraitKind::Warmth,
        bhava::traits::TraitLevel::Low,
    );

    c.bench_function("npc_inspector_summary", |b| {
        b.iter(|| black_box(npc.inspector_summary()));
    });
}

fn bench_npc_compatibility(c: &mut Criterion) {
    let mut a = NpcPersonality::new("A");
    let mut b = NpcPersonality::new("B");
    a.set_trait(
        bhava::traits::TraitKind::Humor,
        bhava::traits::TraitLevel::Highest,
    );
    b.set_trait(
        bhava::traits::TraitKind::Humor,
        bhava::traits::TraitLevel::Lowest,
    );

    c.bench_function("npc_compatibility", |b_iter| {
        b_iter.iter(|| black_box(a.compatibility(&b)));
    });
}

fn bench_npc_blend(c: &mut Criterion) {
    let mut a = NpcPersonality::new("A");
    let mut b = NpcPersonality::new("B");
    a.set_trait(
        bhava::traits::TraitKind::Confidence,
        bhava::traits::TraitLevel::Highest,
    );
    b.set_trait(
        bhava::traits::TraitKind::Patience,
        bhava::traits::TraitLevel::Highest,
    );

    c.bench_function("npc_blend", |b_iter| {
        b_iter.iter(|| black_box(a.blend(&b, 0.5)));
    });
}

criterion_group!(
    personality_benches,
    bench_npc_personality_new,
    bench_npc_personality_inspector_summary,
    bench_npc_compatibility,
    bench_npc_blend,
);

// ---------------------------------------------------------------------------
// Texture benchmarks
// ---------------------------------------------------------------------------

fn bench_average_color(c: &mut Criterion) {
    let data = vec![128u8; 256 * 256 * 4];
    c.bench_function("texture_average_color_256x256", |b| {
        b.iter(|| black_box(salai::texture::average_color(&data, 256, 256)));
    });
}

fn bench_thumbnail_256(c: &mut Criterion) {
    let data = vec![128u8; 512 * 512 * 4];
    c.bench_function("texture_thumbnail_512_to_128", |b| {
        b.iter(|| {
            black_box(salai::texture::generate_thumbnail(
                data.clone(),
                512,
                512,
                128,
            ))
        });
    });
}

fn bench_inspect_texture(c: &mut Criterion) {
    let data = vec![100u8; 64 * 64 * 4];
    c.bench_function("texture_inspect_64x64", |b| {
        b.iter(|| black_box(salai::texture::inspect_texture(&data, 64, 64)));
    });
}

criterion_group!(
    texture_benches,
    bench_average_color,
    bench_thumbnail_256,
    bench_inspect_texture
);

// ---------------------------------------------------------------------------
// Audio benchmarks
// ---------------------------------------------------------------------------

fn bench_inspect_audio(c: &mut Criterion) {
    let samples: Vec<f32> = (0..44100)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
        .collect();
    let buf = dhvani::AudioBuffer::from_interleaved(samples, 1, 44100).unwrap();

    c.bench_function("audio_inspect_1sec", |b| {
        b.iter(|| black_box(salai::audio::inspect_audio(&buf)));
    });
}

fn bench_audio_waveform(c: &mut Criterion) {
    let samples: Vec<f32> = (0..44100)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
        .collect();
    let buf = dhvani::AudioBuffer::from_interleaved(samples, 1, 44100).unwrap();

    c.bench_function("audio_waveform_1sec_100pps", |b| {
        b.iter(|| black_box(salai::audio::waveform(&buf, 100)));
    });
}

criterion_group!(audio_benches, bench_inspect_audio, bench_audio_waveform);

criterion_main!(
    editor_benches,
    inspector_benches,
    hierarchy_benches,
    viewport_benches,
    expr_benches,
    hw_benches,
    history_benches,
    personality_benches,
    texture_benches,
    audio_benches,
);
