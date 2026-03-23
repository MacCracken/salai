//! Salai — game editor for the Kiran engine
//!
//! Visual editor for creating and editing game scenes, inspecting entities,
//! and live-previewing gameplay. Built with egui + kiran + soorat.

pub mod editor;
pub mod hierarchy;
pub mod inspector;
pub mod viewport;

pub use editor::{EditorApp, EditorState};
