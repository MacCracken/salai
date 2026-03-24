//! Core editor application state and layout.

use kiran::World;
use kiran::world::GameClock;
use serde::{Deserialize, Serialize};

/// Editor play state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PlayState {
    /// Editor mode — scene is paused, entities are editable.
    #[default]
    Editing,
    /// Play mode — simulation running.
    Playing,
    /// Paused — simulation frozen, can step.
    Paused,
}

/// Persistent editor state.
#[derive(Debug, Serialize, Deserialize)]
pub struct EditorState {
    /// Current play state.
    pub play_state: PlayState,
    /// Selected entities. Stores raw u64 IDs (index + generation encoded).
    /// First entry is the primary selection.
    selected_entities: Vec<u64>,
    /// Whether the inspector panel is open.
    pub show_inspector: bool,
    /// Whether the hierarchy panel is open.
    pub show_hierarchy: bool,
    /// Whether the viewport is active.
    pub show_viewport: bool,
    /// Scene file path (if loaded).
    pub scene_path: Option<String>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            play_state: PlayState::Editing,
            selected_entities: Vec::new(),
            show_inspector: true,
            show_hierarchy: true,
            show_viewport: true,
            scene_path: None,
        }
    }
}

impl EditorState {
    /// Select a single entity (replaces any existing selection).
    pub fn select(&mut self, entity: kiran::Entity) {
        self.selected_entities.clear();
        self.selected_entities.push(entity.id());
    }

    /// Add an entity to the selection (shift-click).
    pub fn select_add(&mut self, entity: kiran::Entity) {
        let id = entity.id();
        if !self.selected_entities.contains(&id) {
            self.selected_entities.push(id);
        }
    }

    /// Toggle an entity in the selection (ctrl-click).
    pub fn select_toggle(&mut self, entity: kiran::Entity) {
        let id = entity.id();
        if let Some(pos) = self.selected_entities.iter().position(|&e| e == id) {
            self.selected_entities.remove(pos);
        } else {
            self.selected_entities.push(id);
        }
    }

    /// Clear the selection.
    pub fn deselect(&mut self) {
        self.selected_entities.clear();
    }

    /// Get the primary selected entity (first in selection).
    #[must_use]
    pub fn selected(&self) -> Option<kiran::Entity> {
        self.selected_entities
            .first()
            .map(|&id| kiran::Entity::from_id(id))
    }

    /// Get all selected entities.
    #[must_use]
    pub fn selected_all(&self) -> Vec<kiran::Entity> {
        self.selected_entities
            .iter()
            .map(|&id| kiran::Entity::from_id(id))
            .collect()
    }

    /// Number of selected entities.
    #[must_use]
    #[inline]
    pub fn selection_count(&self) -> usize {
        self.selected_entities.len()
    }

    /// Check if a specific entity is selected.
    #[must_use]
    pub fn is_selected(&self, entity: kiran::Entity) -> bool {
        self.selected_entities.contains(&entity.id())
    }

    /// Toggle play/pause.
    pub fn toggle_play(&mut self) {
        self.play_state = match self.play_state {
            PlayState::Editing => PlayState::Playing,
            PlayState::Playing => PlayState::Paused,
            PlayState::Paused => PlayState::Playing,
        };
    }

    /// Stop and return to editing.
    pub fn stop(&mut self) {
        self.play_state = PlayState::Editing;
    }

    /// Is the simulation running?
    #[must_use]
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.play_state == PlayState::Playing
    }
}

/// The main editor application.
pub struct EditorApp {
    pub state: EditorState,
    pub world: World,
    /// Tracked entity list for hierarchy display. Updated on spawn/despawn/load.
    pub(crate) tracked_entities: Vec<kiran::Entity>,
}

impl EditorApp {
    #[must_use]
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(GameClock::with_timestep(1.0 / 60.0));

        Self {
            state: EditorState::default(),
            world,
            tracked_entities: Vec::new(),
        }
    }

    /// Load a scene file into the editor.
    pub fn load_scene(&mut self, path: &str) -> anyhow::Result<()> {
        let toml_str = std::fs::read_to_string(path)?;
        let scene = kiran::scene::load_scene(&toml_str)?;
        let spawned = kiran::scene::spawn_scene(&mut self.world, &scene)?;
        self.tracked_entities.extend(spawned);
        self.state.scene_path = Some(path.to_string());
        tracing::info!(
            path,
            entities = self.tracked_entities.len(),
            "scene loaded in editor"
        );
        Ok(())
    }

    /// Spawn an entity and track it.
    #[must_use]
    pub fn spawn_entity(&mut self) -> kiran::Entity {
        let entity = self.world.spawn();
        self.tracked_entities.push(entity);
        entity
    }

    /// Step the simulation by one frame (when paused).
    pub fn step_frame(&mut self) {
        if self.state.play_state == PlayState::Paused {
            let clock = self.world.get_resource_mut::<GameClock>().unwrap();
            clock.tick(1.0 / 60.0);
        }
    }

    /// Entity count in the world.
    #[must_use]
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.world.entity_count()
    }

    /// Despawn an entity and remove it from tracking and selection.
    pub fn despawn_entity(&mut self, entity: kiran::Entity) -> anyhow::Result<()> {
        self.world.despawn(entity)?;
        self.tracked_entities.retain(|&e| e != entity);
        self.state.selected_entities.retain(|&id| id != entity.id());
        Ok(())
    }

    /// Get the list of tracked entities (alive ones only).
    #[must_use]
    pub fn entities(&self) -> &[kiran::Entity] {
        &self.tracked_entities
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_state_default() {
        let state = EditorState::default();
        assert_eq!(state.play_state, PlayState::Editing);
        assert!(state.selected_entities.is_empty());
        assert!(state.show_inspector);
        assert!(state.show_hierarchy);
    }

    #[test]
    fn editor_state_select_deselect() {
        let mut state = EditorState::default();
        let entity = kiran::Entity::new(5, 2);

        state.select(entity);
        assert_eq!(state.selected(), Some(entity));

        state.deselect();
        assert!(state.selected().is_none());
    }

    #[test]
    fn editor_state_toggle_play() {
        let mut state = EditorState::default();

        state.toggle_play();
        assert_eq!(state.play_state, PlayState::Playing);
        assert!(state.is_playing());

        state.toggle_play();
        assert_eq!(state.play_state, PlayState::Paused);

        state.toggle_play();
        assert_eq!(state.play_state, PlayState::Playing);

        state.stop();
        assert_eq!(state.play_state, PlayState::Editing);
    }

    #[test]
    fn editor_app_new() {
        let app = EditorApp::new();
        assert_eq!(app.entity_count(), 0);
        assert!(!app.state.is_playing());
    }

    #[test]
    fn editor_app_step_frame() {
        let mut app = EditorApp::new();
        app.state.play_state = PlayState::Paused;
        app.step_frame();

        let clock = app.world.get_resource::<GameClock>().unwrap();
        assert_eq!(clock.frame, 1);
    }

    #[test]
    fn play_state_serde() {
        let state = PlayState::Playing;
        let json = serde_json::to_string(&state).unwrap();
        let decoded: PlayState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, decoded);
    }

    #[test]
    fn editor_state_serde() {
        let state = EditorState {
            scene_path: Some("test.toml".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&state).unwrap();
        let decoded: EditorState = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.scene_path.as_deref(), Some("test.toml"));
    }

    #[test]
    fn step_frame_noop_when_not_paused() {
        let mut app = EditorApp::new();
        // In Editing state — step_frame should be a no-op
        app.step_frame();
        let clock = app.world.get_resource::<GameClock>().unwrap();
        assert_eq!(clock.frame, 0);

        // In Playing state — step_frame should also be a no-op
        app.state.play_state = PlayState::Playing;
        app.step_frame();
        let clock = app.world.get_resource::<GameClock>().unwrap();
        assert_eq!(clock.frame, 0);
    }

    #[test]
    fn stop_from_all_states() {
        let mut state = EditorState::default();

        // Stop from Editing — stays Editing
        state.stop();
        assert_eq!(state.play_state, PlayState::Editing);

        // Stop from Playing
        state.play_state = PlayState::Playing;
        state.stop();
        assert_eq!(state.play_state, PlayState::Editing);

        // Stop from Paused
        state.play_state = PlayState::Paused;
        state.stop();
        assert_eq!(state.play_state, PlayState::Editing);
    }

    #[test]
    fn load_scene_invalid_path() {
        let mut app = EditorApp::new();
        let result = app.load_scene("/nonexistent/path/scene.toml");
        assert!(result.is_err());
        assert!(app.state.scene_path.is_none());
    }

    #[test]
    fn editor_state_panel_toggles() {
        let mut state = EditorState::default();
        assert!(state.show_inspector);
        assert!(state.show_hierarchy);
        assert!(state.show_viewport);

        state.show_inspector = false;
        state.show_hierarchy = false;
        state.show_viewport = false;

        // Serde round-trip preserves toggle state
        let json = serde_json::to_string(&state).unwrap();
        let decoded: EditorState = serde_json::from_str(&json).unwrap();
        assert!(!decoded.show_inspector);
        assert!(!decoded.show_hierarchy);
        assert!(!decoded.show_viewport);
    }

    #[test]
    fn selected_entity_preserves_generation() {
        let mut state = EditorState::default();
        let entity = kiran::Entity::new(5, 7);
        state.select(entity);
        let selected = state.selected().unwrap();
        assert_eq!(selected.index(), 5);
        assert_eq!(selected.generation(), 7);
    }

    #[test]
    fn editor_app_default_trait() {
        let app = EditorApp::default();
        assert_eq!(app.entity_count(), 0);
        assert_eq!(app.state.play_state, PlayState::Editing);
    }

    #[test]
    fn play_state_all_variants_serde() {
        for state in [PlayState::Editing, PlayState::Playing, PlayState::Paused] {
            let json = serde_json::to_string(&state).unwrap();
            let decoded: PlayState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, decoded);
        }
    }

    #[test]
    fn editor_state_serde_with_selection() {
        let mut state = EditorState::default();
        let entity = kiran::Entity::new(10, 5);
        state.select(entity);
        state.play_state = PlayState::Paused;
        state.show_inspector = false;
        state.scene_path = Some("level.toml".into());

        let json = serde_json::to_string(&state).unwrap();
        let decoded: EditorState = serde_json::from_str(&json).unwrap();

        let sel = decoded.selected().unwrap();
        assert_eq!(sel.index(), 10);
        assert_eq!(sel.generation(), 5);
        assert_eq!(decoded.play_state, PlayState::Paused);
        assert!(!decoded.show_inspector);
        assert_eq!(decoded.scene_path.as_deref(), Some("level.toml"));
    }

    #[test]
    fn editor_app_spawn_and_count() {
        let mut app = EditorApp::new();
        app.world.spawn();
        app.world.spawn();
        app.world.spawn();
        assert_eq!(app.entity_count(), 3);
    }

    #[test]
    fn spawn_entity_tracks() {
        let mut app = EditorApp::new();
        let e1 = app.spawn_entity();
        let e2 = app.spawn_entity();
        assert_eq!(app.entities().len(), 2);
        assert_eq!(app.entity_count(), 2);
        assert!(app.entities().contains(&e1));
        assert!(app.entities().contains(&e2));
    }

    #[test]
    fn despawn_entity_untracks() {
        let mut app = EditorApp::new();
        let e = app.spawn_entity();
        assert_eq!(app.entities().len(), 1);
        app.despawn_entity(e).unwrap();
        assert!(app.entities().is_empty());
        assert_eq!(app.entity_count(), 0);
    }

    #[test]
    fn despawn_clears_selection() {
        let mut app = EditorApp::new();
        let e = app.spawn_entity();
        app.state.select(e);
        assert_eq!(app.state.selected(), Some(e));
        app.despawn_entity(e).unwrap();
        assert!(app.state.selected().is_none());
    }

    #[test]
    fn despawn_invalid_entity() {
        let mut app = EditorApp::new();
        let fake = kiran::Entity::new(999, 0);
        assert!(app.despawn_entity(fake).is_err());
    }

    #[test]
    fn select_replaces_previous() {
        let mut state = EditorState::default();
        let e1 = kiran::Entity::new(1, 0);
        let e2 = kiran::Entity::new(2, 0);
        state.select(e1);
        state.select(e2);
        assert_eq!(state.selected(), Some(e2));
        assert_eq!(state.selection_count(), 1);
    }

    #[test]
    fn select_add_multi() {
        let mut state = EditorState::default();
        let e1 = kiran::Entity::new(1, 0);
        let e2 = kiran::Entity::new(2, 0);
        let e3 = kiran::Entity::new(3, 0);
        state.select(e1);
        state.select_add(e2);
        state.select_add(e3);
        assert_eq!(state.selection_count(), 3);
        assert_eq!(state.selected(), Some(e1)); // primary is first
        assert!(state.is_selected(e2));
        assert!(state.is_selected(e3));
    }

    #[test]
    fn select_add_no_duplicates() {
        let mut state = EditorState::default();
        let e = kiran::Entity::new(1, 0);
        state.select(e);
        state.select_add(e);
        assert_eq!(state.selection_count(), 1);
    }

    #[test]
    fn select_toggle() {
        let mut state = EditorState::default();
        let e1 = kiran::Entity::new(1, 0);
        let e2 = kiran::Entity::new(2, 0);
        state.select(e1);
        state.select_add(e2);
        assert_eq!(state.selection_count(), 2);

        // Toggle off e1
        state.select_toggle(e1);
        assert_eq!(state.selection_count(), 1);
        assert!(!state.is_selected(e1));
        assert!(state.is_selected(e2));

        // Toggle e1 back on
        state.select_toggle(e1);
        assert_eq!(state.selection_count(), 2);
    }

    #[test]
    fn selected_all() {
        let mut state = EditorState::default();
        let e1 = kiran::Entity::new(1, 0);
        let e2 = kiran::Entity::new(2, 0);
        state.select(e1);
        state.select_add(e2);
        let all = state.selected_all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&e1));
        assert!(all.contains(&e2));
    }

    #[test]
    fn despawn_removes_from_multi_selection() {
        let mut app = EditorApp::new();
        let e1 = app.spawn_entity();
        let e2 = app.spawn_entity();
        app.state.select(e1);
        app.state.select_add(e2);
        assert_eq!(app.state.selection_count(), 2);

        app.despawn_entity(e1).unwrap();
        assert_eq!(app.state.selection_count(), 1);
        assert!(!app.state.is_selected(e1));
        assert!(app.state.is_selected(e2));
    }
}
