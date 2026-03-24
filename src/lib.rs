//! Salai — game editor for the Kiran engine
//!
//! Visual editor for creating and editing game scenes, inspecting entities,
//! and live-previewing gameplay. Built with egui + kiran + soorat.
//!
//! Editor primitives (expression eval, hardware detection, undo/redo, hierarchy,
//! inspector) are provided by [`muharrir`]. Salai adds game-specific modules
//! on top: ECS inspector, entity hierarchy, viewport, personality editing.

pub mod audio;
pub mod editor;
pub mod hierarchy;
pub mod inspector;
pub mod personality;
pub mod picking;
pub mod scene_edit;
pub mod texture;
pub mod ui;
pub mod viewport;
pub mod viewport_renderer;

// Re-export muharrir primitives used by salai consumers
pub use muharrir::expr::{self, ExprError, eval_f64, eval_or, eval_or_parse};
pub use muharrir::history::{self, Action, History};
pub use muharrir::hw::{self, HardwareProfile, QualityTier};

pub use audio::AudioInfo;
pub use ui::animation_panel::{AnimClipState, AnimationEditor};
pub use ui::terrain_panel::{BrushTool, TerrainEditor};
pub use editor::{EditorApp, EditorState, PlayState};
pub use hierarchy::{HierarchyNode, build_hierarchy, flatten_hierarchy};
pub use inspector::{ComponentInfo, inspect_entity};
pub use personality::{NpcPersonality, PersonalitySummary};
pub use texture::TextureInfo;
pub use ui::SalaiApp;
pub use viewport::{GizmoMode, ViewportState};
