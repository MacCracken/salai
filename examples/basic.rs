//! Basic salai usage — create editor, spawn entities, inspect, and save.

fn main() {
    // Create a new editor
    let mut app = salai::EditorApp::new();
    println!("Salai editor — {} entities", app.entity_count());

    // Spawn entities
    let player = app.spawn_entity();
    app.world
        .insert_component(player, kiran::scene::Name("Player".into()))
        .unwrap();
    app.world
        .insert_component(
            player,
            kiran::scene::Position(hisab::Vec3::new(0.0, 1.0, 0.0)),
        )
        .unwrap();

    let light = app.spawn_entity();
    app.world
        .insert_component(light, kiran::scene::Name("Sun".into()))
        .unwrap();
    app.world
        .insert_component(light, kiran::scene::LightComponent { intensity: 1.0 })
        .unwrap();

    println!("Spawned {} entities", app.entity_count());

    // Inspect an entity
    let info = salai::inspect_entity(&app.world, player);
    println!("\nInspecting Player:");
    for comp in &info {
        println!("  {}: {}", comp.name, comp.details);
    }

    // Build hierarchy
    let tree = salai::build_hierarchy(&app.world, app.entities());
    let flat = salai::flatten_hierarchy(&tree);
    println!("\nHierarchy:");
    for (depth, _entity, name) in &flat {
        println!("  {}{}", "  ".repeat(*depth), name);
    }

    // Expression evaluation
    let val = salai::eval_f64("2 * pi + sqrt(9)").unwrap();
    println!("\nExpression: 2 * pi + sqrt(9) = {val:.4}");

    // Extract and display scene TOML
    let scene = salai::scene_edit::extract_scene(&app.world, app.entities(), "Example Scene");
    let toml = salai::scene_edit::scene_to_toml(&scene).unwrap();
    println!("\nScene TOML:\n{toml}");
}
