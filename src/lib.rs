//! Salai — game editor for the Kiran engine
//!
//! Visual editor for creating and editing game scenes, inspecting entities,
//! and live-previewing gameplay. Built with egui + kiran + soorat.

pub mod editor;
pub mod expr;
pub mod hierarchy;
pub mod hw;
pub mod inspector;
pub mod viewport;

pub use editor::{EditorApp, EditorState, PlayState};
pub use expr::{ExprError, eval_f64, eval_or, eval_or_parse};
pub use hierarchy::{HierarchyNode, build_hierarchy, flatten_hierarchy};
pub use hw::{HardwareProfile, QualityTier};
pub use inspector::{ComponentInfo, inspect_entity};
pub use viewport::{GizmoMode, ViewportState};
