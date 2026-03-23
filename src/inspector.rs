//! Entity inspector panel — view and edit components on the selected entity.

use kiran::World;
use kiran::scene::{LightComponent, Material, Name, Position, Tags};

/// Component info for display in the inspector.
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub name: &'static str,
    pub details: String,
}

/// Gather component information for an entity.
pub fn inspect_entity(world: &World, entity: kiran::Entity) -> Vec<ComponentInfo> {
    let mut components = Vec::new();

    if let Some(name) = world.get_component::<Name>(entity) {
        components.push(ComponentInfo {
            name: "Name",
            details: name.0.clone(),
        });
    }

    if let Some(pos) = world.get_component::<Position>(entity) {
        components.push(ComponentInfo {
            name: "Position",
            details: format!("({:.2}, {:.2}, {:.2})", pos.0.x, pos.0.y, pos.0.z),
        });
    }

    if let Some(light) = world.get_component::<LightComponent>(entity) {
        components.push(ComponentInfo {
            name: "Light",
            details: format!("intensity: {:.2}", light.intensity),
        });
    }

    if let Some(tags) = world.get_component::<Tags>(entity) {
        components.push(ComponentInfo {
            name: "Tags",
            details: tags.0.join(", "),
        });
    }

    if let Some(mat) = world.get_component::<Material>(entity) {
        let tex = mat.texture.as_deref().unwrap_or("none");
        components.push(ComponentInfo {
            name: "Material",
            details: format!(
                "color: [{:.2}, {:.2}, {:.2}, {:.2}] tex: {}",
                mat.color[0], mat.color[1], mat.color[2], mat.color[3], tex
            ),
        });
    }

    components
}

#[cfg(test)]
mod tests {
    use super::*;
    use hisab::Vec3;

    #[test]
    fn inspect_empty_entity() {
        let mut world = World::new();
        let e = world.spawn();
        let info = inspect_entity(&world, e);
        assert!(info.is_empty());
    }

    #[test]
    fn inspect_entity_with_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Name("Player".into())).unwrap();
        world
            .insert_component(e, Position(Vec3::new(1.0, 2.0, 3.0)))
            .unwrap();
        world
            .insert_component(e, LightComponent { intensity: 0.8 })
            .unwrap();
        world
            .insert_component(e, Tags(vec!["hero".into(), "controllable".into()]))
            .unwrap();

        let info = inspect_entity(&world, e);
        assert_eq!(info.len(), 4);
        assert_eq!(info[0].name, "Name");
        assert!(info[0].details.contains("Player"));
        assert_eq!(info[1].name, "Position");
        assert!(info[1].details.contains("1.00"));
    }

    #[test]
    fn inspect_entity_with_material() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(
                e,
                Material {
                    color: [1.0, 0.0, 0.0, 1.0],
                    texture: Some("brick.png".into()),
                },
            )
            .unwrap();

        let info = inspect_entity(&world, e);
        assert_eq!(info.len(), 1);
        assert!(info[0].details.contains("brick.png"));
    }

    #[test]
    fn inspect_dead_entity() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Name("Gone".into())).unwrap();
        world.despawn(e).unwrap();

        let info = inspect_entity(&world, e);
        assert!(info.is_empty());
    }

    #[test]
    fn inspect_entity_all_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Name("Full".into())).unwrap();
        world
            .insert_component(e, Position(Vec3::new(1.0, 2.0, 3.0)))
            .unwrap();
        world
            .insert_component(e, LightComponent { intensity: 0.5 })
            .unwrap();
        world
            .insert_component(e, Tags(vec!["a".into(), "b".into()]))
            .unwrap();
        world
            .insert_component(
                e,
                Material {
                    color: [0.1, 0.2, 0.3, 1.0],
                    texture: None,
                },
            )
            .unwrap();

        let info = inspect_entity(&world, e);
        assert_eq!(info.len(), 5);

        let names: Vec<&str> = info.iter().map(|i| i.name).collect();
        assert!(names.contains(&"Name"));
        assert!(names.contains(&"Position"));
        assert!(names.contains(&"Light"));
        assert!(names.contains(&"Tags"));
        assert!(names.contains(&"Material"));
    }
}
