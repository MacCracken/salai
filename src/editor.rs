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
    /// Currently selected entity (if any).
    pub selected_entity: Option<u64>,
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
    /// Select an entity by its raw id.
    pub fn select(&mut self, entity: kiran::Entity) {
        self.selected_entity = Some(entity.id());
    }

    /// Clear the selection.
    pub fn deselect(&mut self) {
        self.selected_entity = None;
    }

    /// Get the selected entity.
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
}
