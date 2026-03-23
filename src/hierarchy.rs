//! Scene hierarchy tree — displays parent-child entity relationships.

use kiran::World;
use kiran::scene::{Children, Name, Parent};

/// A node in the hierarchy tree.
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub entity: kiran::Entity,
    pub name: String,
    pub children: Vec<HierarchyNode>,
    pub depth: usize,
}

/// Build the hierarchy tree from the world.
/// Returns only root entities (those without a Parent component).
pub fn build_hierarchy(world: &World, entities: &[kiran::Entity]) -> Vec<HierarchyNode> {
    let mut roots = Vec::new();

    for &entity in entities {
        if !world.is_alive(entity) {
            continue;
        }
        // Only include root entities (no parent)
        if world.has_component::<Parent>(entity) {
            continue;
        }
        roots.push(build_node(world, entity, 0));
    }

    roots
}

fn build_node(world: &World, entity: kiran::Entity, depth: usize) -> HierarchyNode {
    let name = world
        .get_component::<Name>(entity)
        .map(|n| n.0.clone())
        .unwrap_or_else(|| format!("Entity {}", entity));

    let children = if let Some(children_comp) = world.get_component::<Children>(entity) {
        children_comp
            .0
            .iter()
            .filter(|&&e| world.is_alive(e))
            .map(|&e| build_node(world, e, depth + 1))
            .collect()
    } else {
        Vec::new()
    };

    HierarchyNode {
        entity,
        name,
        children,
        depth,
    }
}

/// Flatten a hierarchy tree into a depth-first list for display.
pub fn flatten_hierarchy(nodes: &[HierarchyNode]) -> Vec<(usize, kiran::Entity, &str)> {
    let mut result = Vec::new();
    for node in nodes {
        flatten_node(node, &mut result);
    }
    result
}

fn flatten_node<'a>(node: &'a HierarchyNode, result: &mut Vec<(usize, kiran::Entity, &'a str)>) {
    result.push((node.depth, node.entity, &node.name));
    for child in &node.children {
        flatten_node(child, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kiran::scene::set_parent;

    #[test]
    fn hierarchy_flat_entities() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert_component(e1, Name("A".into())).unwrap();
        world.insert_component(e2, Name("B".into())).unwrap();

        let tree = build_hierarchy(&world, &[e1, e2]);
        assert_eq!(tree.len(), 2);
        assert!(tree[0].children.is_empty());
    }

    #[test]
    fn hierarchy_parent_child() {
        let mut world = World::new();
        let parent = world.spawn();
        let child = world.spawn();
        world
            .insert_component(parent, Name("Parent".into()))
            .unwrap();
        world.insert_component(child, Name("Child".into())).unwrap();
        set_parent(&mut world, child, parent).unwrap();

        let tree = build_hierarchy(&world, &[parent, child]);
        assert_eq!(tree.len(), 1); // only root
        assert_eq!(tree[0].name, "Parent");
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].name, "Child");
        assert_eq!(tree[0].children[0].depth, 1);
    }

    #[test]
    fn hierarchy_deep_nesting() {
        let mut world = World::new();
        let root = world.spawn();
        let l1 = world.spawn();
        let l2 = world.spawn();
        world.insert_component(root, Name("Root".into())).unwrap();
        world.insert_component(l1, Name("L1".into())).unwrap();
        world.insert_component(l2, Name("L2".into())).unwrap();
        set_parent(&mut world, l1, root).unwrap();
        set_parent(&mut world, l2, l1).unwrap();

        let tree = build_hierarchy(&world, &[root, l1, l2]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children[0].children[0].name, "L2");
        assert_eq!(tree[0].children[0].children[0].depth, 2);
    }

    #[test]
    fn flatten_hierarchy_order() {
        let mut world = World::new();
        let root = world.spawn();
        let c1 = world.spawn();
        let c2 = world.spawn();
        world.insert_component(root, Name("Root".into())).unwrap();
        world.insert_component(c1, Name("C1".into())).unwrap();
        world.insert_component(c2, Name("C2".into())).unwrap();
        set_parent(&mut world, c1, root).unwrap();
        set_parent(&mut world, c2, root).unwrap();

        let tree = build_hierarchy(&world, &[root, c1, c2]);
        let flat = flatten_hierarchy(&tree);
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].2, "Root");
        assert_eq!(flat[0].0, 0); // depth 0
        assert_eq!(flat[1].0, 1); // depth 1
        assert_eq!(flat[2].0, 1); // depth 1
    }

    #[test]
    fn hierarchy_dead_entity_filtered() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert_component(e1, Name("Alive".into())).unwrap();
        world.insert_component(e2, Name("Dead".into())).unwrap();
        world.despawn(e2).unwrap();

        let tree = build_hierarchy(&world, &[e1, e2]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "Alive");
    }

    #[test]
    fn hierarchy_unnamed_entity() {
        let mut world = World::new();
        let e = world.spawn();
        // No Name component

        let tree = build_hierarchy(&world, &[e]);
        assert_eq!(tree.len(), 1);
        assert!(tree[0].name.contains("Entity"));
    }
}
