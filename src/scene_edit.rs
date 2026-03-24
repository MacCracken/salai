//! Scene editing — entity CRUD, component editing, and scene serialization.
//!
//! All mutations are recorded in the undo/redo history via [`muharrir::History`].

use kiran::World;
use kiran::scene::{EntityDef, LightComponent, Material, Name, Position, SceneDefinition, Tags};
use muharrir::history::{Action, History};

/// Add an entity to the world and record the action in history.
pub fn add_entity(
    world: &mut World,
    tracked: &mut Vec<kiran::Entity>,
    history: &mut History,
    name: &str,
) -> kiran::Entity {
    let entity = world.spawn();
    world
        .insert_component(entity, Name(name.to_string()))
        .unwrap();
    tracked.push(entity);

    history.record(
        "scene",
        Action::new(
            "add_entity",
            serde_json::json!({ "entity": entity.id(), "name": name }),
        ),
    );
    tracing::info!(entity = %entity, name, "entity added");
    entity
}

/// Set the position of an entity and record the change.
pub fn set_position(
    world: &mut World,
    entity: kiran::Entity,
    position: hisab::Vec3,
    history: &mut History,
) {
    let before = world
        .get_component::<Position>(entity)
        .map(|p| [p.0.x, p.0.y, p.0.z])
        .unwrap_or([0.0, 0.0, 0.0]);

    world.insert_component(entity, Position(position)).unwrap();

    history.record(
        "inspector",
        Action::new(
            "set_position",
            serde_json::json!({
                "entity": entity.id(),
                "before": before,
                "after": [position.x, position.y, position.z]
            }),
        ),
    );
}

/// Set the name of an entity and record the change.
pub fn set_name(world: &mut World, entity: kiran::Entity, new_name: &str, history: &mut History) {
    let before = world
        .get_component::<Name>(entity)
        .map(|n| n.0.clone())
        .unwrap_or_default();

    world
        .insert_component(entity, Name(new_name.to_string()))
        .unwrap();

    history.record(
        "inspector",
        Action::new(
            "set_name",
            serde_json::json!({
                "entity": entity.id(),
                "before": before,
                "after": new_name
            }),
        ),
    );
}

/// Set the light intensity of an entity and record the change.
pub fn set_light_intensity(
    world: &mut World,
    entity: kiran::Entity,
    intensity: f32,
    history: &mut History,
) {
    let before = world
        .get_component::<LightComponent>(entity)
        .map(|l| l.intensity);

    world
        .insert_component(entity, LightComponent { intensity })
        .unwrap();

    history.record(
        "inspector",
        Action::new(
            "set_light_intensity",
            serde_json::json!({
                "entity": entity.id(),
                "before": before,
                "after": intensity
            }),
        ),
    );
}

/// Extract the current scene state from the world into a serializable SceneDefinition.
#[must_use]
pub fn extract_scene(
    world: &World,
    entities: &[kiran::Entity],
    scene_name: &str,
) -> SceneDefinition {
    let entity_defs: Vec<EntityDef> = entities
        .iter()
        .filter(|&&e| world.is_alive(e) && !world.has_component::<kiran::scene::Parent>(e))
        .map(|&e| extract_entity_def(world, e))
        .collect();

    SceneDefinition {
        name: scene_name.to_string(),
        description: String::new(),
        prefabs: Vec::new(),
        entities: entity_defs,
    }
}

/// Extract a single entity (and its children) into an EntityDef.
fn extract_entity_def(world: &World, entity: kiran::Entity) -> EntityDef {
    let name = world
        .get_component::<Name>(entity)
        .map(|n| n.0.clone())
        .unwrap_or_else(|| format!("Entity {entity}"));

    let position = world
        .get_component::<Position>(entity)
        .map(|p| [p.0.x, p.0.y, p.0.z])
        .unwrap_or([0.0, 0.0, 0.0]);

    let light_intensity = world
        .get_component::<LightComponent>(entity)
        .map(|l| l.intensity);

    let tags = world
        .get_component::<Tags>(entity)
        .map(|t| t.0.clone())
        .unwrap_or_default();

    let material = world.get_component::<Material>(entity).cloned();

    let children = world
        .get_component::<kiran::scene::Children>(entity)
        .map(|ch| {
            ch.0.iter()
                .filter(|&&e| world.is_alive(e))
                .map(|&e| extract_entity_def(world, e))
                .collect()
        })
        .unwrap_or_default();

    EntityDef {
        name,
        position,
        light_intensity,
        tags,
        material,
        children,
        prefab: None,
        sound: None,
        physics: None,
    }
}

/// Serialize a scene to TOML string.
pub fn scene_to_toml(scene: &SceneDefinition) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(scene)
}

/// Save a scene to a file.
pub fn save_scene(scene: &SceneDefinition, path: &str) -> anyhow::Result<()> {
    let toml_str = scene_to_toml(scene)?;
    std::fs::write(path, toml_str)?;
    tracing::info!(path, entities = scene.entities.len(), "scene saved");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_entity_records_history() {
        let mut world = World::new();
        let mut tracked = Vec::new();
        let mut history = History::new();

        let e = add_entity(&mut world, &mut tracked, &mut history, "Player");
        assert!(world.is_alive(e));
        assert_eq!(tracked.len(), 1);
        assert_eq!(history.len(), 1);

        let name = world.get_component::<Name>(e).unwrap();
        assert_eq!(name.0, "Player");
    }

    #[test]
    fn set_position_records_before_after() {
        let mut world = World::new();
        let mut history = History::new();
        let e = world.spawn();
        world
            .insert_component(e, Position(hisab::Vec3::ZERO))
            .unwrap();

        set_position(&mut world, e, hisab::Vec3::new(1.0, 2.0, 3.0), &mut history);

        let pos = world.get_component::<Position>(e).unwrap();
        assert_eq!(pos.0.x, 1.0);
        assert_eq!(pos.0.y, 2.0);
        assert_eq!(pos.0.z, 3.0);

        assert_eq!(history.len(), 1);
        let entry = &history.entries()[0];
        assert_eq!(entry.action(), "set_position");
    }

    #[test]
    fn set_name_records_change() {
        let mut world = World::new();
        let mut history = History::new();
        let e = world.spawn();
        world.insert_component(e, Name("Old".into())).unwrap();

        set_name(&mut world, e, "New", &mut history);

        let name = world.get_component::<Name>(e).unwrap();
        assert_eq!(name.0, "New");
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn set_light_intensity_records_change() {
        let mut world = World::new();
        let mut history = History::new();
        let e = world.spawn();

        set_light_intensity(&mut world, e, 0.8, &mut history);

        let light = world.get_component::<LightComponent>(e).unwrap();
        assert_eq!(light.intensity, 0.8);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn extract_scene_basic() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Name("Hero".into())).unwrap();
        world
            .insert_component(e, Position(hisab::Vec3::new(1.0, 0.0, 0.0)))
            .unwrap();
        world
            .insert_component(e, Tags(vec!["player".into()]))
            .unwrap();

        let scene = extract_scene(&world, &[e], "TestScene");
        assert_eq!(scene.name, "TestScene");
        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].name, "Hero");
        assert_eq!(scene.entities[0].position, [1.0, 0.0, 0.0]);
        assert_eq!(scene.entities[0].tags, vec!["player"]);
    }

    #[test]
    fn extract_scene_with_children() {
        let mut world = World::new();
        let parent = world.spawn();
        let child = world.spawn();
        world
            .insert_component(parent, Name("Parent".into()))
            .unwrap();
        world.insert_component(child, Name("Child".into())).unwrap();
        kiran::scene::set_parent(&mut world, child, parent).unwrap();

        let scene = extract_scene(&world, &[parent, child], "Hierarchy");
        assert_eq!(scene.entities.len(), 1); // only root
        assert_eq!(scene.entities[0].children.len(), 1);
        assert_eq!(scene.entities[0].children[0].name, "Child");
    }

    #[test]
    fn scene_to_toml_roundtrip() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Name("Test".into())).unwrap();
        world
            .insert_component(e, Position(hisab::Vec3::new(1.0, 2.0, 3.0)))
            .unwrap();

        let scene = extract_scene(&world, &[e], "RoundTrip");
        let toml_str = scene_to_toml(&scene).unwrap();

        // Parse back
        let loaded = kiran::scene::load_scene(&toml_str).unwrap();
        assert_eq!(loaded.name, "RoundTrip");
        assert_eq!(loaded.entities.len(), 1);
        assert_eq!(loaded.entities[0].name, "Test");
    }

    #[test]
    fn extract_scene_skips_dead() {
        let mut world = World::new();
        let alive = world.spawn();
        let dead = world.spawn();
        world.insert_component(alive, Name("Alive".into())).unwrap();
        world.insert_component(dead, Name("Dead".into())).unwrap();
        world.despawn(dead).unwrap();

        let scene = extract_scene(&world, &[alive, dead], "FilterDead");
        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].name, "Alive");
    }

    #[test]
    fn multiple_edits_undo_sequence() {
        let mut world = World::new();
        let mut tracked = Vec::new();
        let mut history = History::new();

        let e = add_entity(&mut world, &mut tracked, &mut history, "A");
        set_position(&mut world, e, hisab::Vec3::new(5.0, 0.0, 0.0), &mut history);
        set_name(&mut world, e, "B", &mut history);

        assert_eq!(history.len(), 3);

        // Undo all
        let entry = history.undo().unwrap();
        assert_eq!(entry.action(), "set_name");

        let entry = history.undo().unwrap();
        assert_eq!(entry.action(), "set_position");

        let entry = history.undo().unwrap();
        assert_eq!(entry.action(), "add_entity");

        assert!(!history.can_undo());

        // Redo all
        history.redo();
        history.redo();
        history.redo();
        assert!(!history.can_redo());
    }
}
