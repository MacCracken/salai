//! Core editor application state and layout.

use kiran::World;
use kiran::world::GameClock;
use serde::{Deserialize, Serialize};

/// Editor play state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
    /// Currently selected entity (if any). Stores the raw u64 which encodes
    /// both index and generation — prevents stale selection after entity recycling.
    selected_entity: Option<u64>,
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
            selected_entity: None,
            show_inspector: true,
            show_hierarchy: true,
            show_viewport: true,
            scene_path: None,
        }
    }
}

impl EditorState {
    /// Select an entity. Stores the full id (index + generation).
    pub fn select(&mut self, entity: kiran::Entity) {
        self.selected_entity = Some(entity.id());
    }

    /// Clear the selection.
    pub fn deselect(&mut self) {
        self.selected_entity = None;
    }

    /// Get the selected entity, reconstructed with its original generation.
    pub fn selected(&self) -> Option<kiran::Entity> {
        self.selected_entity.map(kiran::Entity::from_id)
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
    pub fn is_playing(&self) -> bool {
        self.play_state == PlayState::Playing
    }
}

/// The main editor application.
pub struct EditorApp {
    pub state: EditorState,
    pub world: World,
}

impl EditorApp {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(GameClock::with_timestep(1.0 / 60.0));

        Self {
            state: EditorState::default(),
            world,
        }
    }

    /// Load a scene file into the editor.
    pub fn load_scene(&mut self, path: &str) -> anyhow::Result<()> {
        let toml_str = std::fs::read_to_string(path)?;
        let scene = kiran::scene::load_scene(&toml_str)?;
        kiran::scene::spawn_scene(&mut self.world, &scene)?;
        self.state.scene_path = Some(path.to_string());
        tracing::info!(path, "scene loaded in editor");
        Ok(())
    }

    /// Step the simulation by one frame (when paused).
    pub fn step_frame(&mut self) {
        if self.state.play_state == PlayState::Paused {
            let clock = self.world.get_resource_mut::<GameClock>().unwrap();
            clock.tick(1.0 / 60.0);
        }
    }

    /// Entity count in the world.
    pub fn entity_count(&self) -> usize {
        self.world.entity_count()
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
        assert!(state.selected_entity.is_none());
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
    fn select_multiple_entities_last_wins() {
        let mut state = EditorState::default();
        let e1 = kiran::Entity::new(1, 0);
        let e2 = kiran::Entity::new(2, 0);
        state.select(e1);
        state.select(e2);
        assert_eq!(state.selected(), Some(e2));
    }
}
