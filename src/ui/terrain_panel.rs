//! Terrain/landscape editing panel — brush tools, heightmap editing, preview.
//!
//! Consumes soorat's `TerrainConfig`, `TerrainData`, and `generate_terrain()`.

use serde::{Deserialize, Serialize};

/// Terrain brush tool type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BrushTool {
    /// Raise terrain height.
    #[default]
    Raise,
    /// Lower terrain height.
    Lower,
    /// Smooth terrain to average of neighbors.
    Smooth,
    /// Flatten terrain to a target height.
    Flatten,
    /// Paint texture/material onto terrain.
    Paint,
}

impl BrushTool {
    /// Display label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            BrushTool::Raise => "Raise",
            BrushTool::Lower => "Lower",
            BrushTool::Smooth => "Smooth",
            BrushTool::Flatten => "Flatten",
            BrushTool::Paint => "Paint",
        }
    }

    /// Icon color for the brush.
    #[must_use]
    pub fn color(self) -> egui::Color32 {
        match self {
            BrushTool::Raise => egui::Color32::from_rgb(80, 200, 80),
            BrushTool::Lower => egui::Color32::from_rgb(200, 80, 80),
            BrushTool::Smooth => egui::Color32::from_rgb(80, 140, 220),
            BrushTool::Flatten => egui::Color32::from_rgb(200, 200, 80),
            BrushTool::Paint => egui::Color32::from_rgb(180, 100, 220),
        }
    }
}

/// Terrain editor state.
pub struct TerrainEditor {
    /// Grid width (cells).
    pub width: u32,
    /// Grid depth (cells).
    pub depth: u32,
    /// Height scale factor.
    pub height_scale: f32,
    /// Heightmap data (row-major, (width+1)*(depth+1) entries).
    pub heights: Vec<f32>,
    /// Currently selected brush tool.
    pub brush: BrushTool,
    /// Brush radius in grid cells.
    pub brush_radius: f32,
    /// Brush strength (0.0 to 1.0).
    pub brush_strength: f32,
    /// Flatten target height.
    pub flatten_height: f32,
    /// Whether the terrain has been modified since last save/generate.
    pub dirty: bool,
}

impl Default for TerrainEditor {
    fn default() -> Self {
        Self::new(64, 64)
    }
}

impl TerrainEditor {
    /// Create a new terrain editor with a flat heightmap.
    #[must_use]
    pub fn new(width: u32, depth: u32) -> Self {
        let size = ((width + 1) * (depth + 1)) as usize;
        Self {
            width,
            depth,
            height_scale: 10.0,
            heights: vec![0.0; size],
            brush: BrushTool::Raise,
            brush_radius: 3.0,
            brush_strength: 0.5,
            flatten_height: 0.0,
            dirty: false,
        }
    }

    /// Apply the current brush at a grid position.
    pub fn apply_brush(&mut self, cx: f32, cz: f32) {
        let cols = self.width + 1;
        let rows = self.depth + 1;
        let r = self.brush_radius;
        let strength = self.brush_strength * 0.1;

        let min_x = ((cx - r).floor().max(0.0)) as u32;
        let max_x = ((cx + r).ceil().min(cols as f32 - 1.0)) as u32;
        let min_z = ((cz - r).floor().max(0.0)) as u32;
        let max_z = ((cz + r).ceil().min(rows as f32 - 1.0)) as u32;

        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let dx = x as f32 - cx;
                let dz = z as f32 - cz;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist > r {
                    continue;
                }
                let falloff = 1.0 - (dist / r);
                let idx = (z * cols + x) as usize;
                if idx >= self.heights.len() {
                    continue;
                }

                match self.brush {
                    BrushTool::Raise => self.heights[idx] += strength * falloff,
                    BrushTool::Lower => self.heights[idx] -= strength * falloff,
                    BrushTool::Smooth => {
                        let avg = self.neighbor_average(x, z);
                        self.heights[idx] += (avg - self.heights[idx]) * strength * falloff;
                    }
                    BrushTool::Flatten => {
                        let diff = self.flatten_height - self.heights[idx];
                        self.heights[idx] += diff * strength * falloff;
                    }
                    BrushTool::Paint => {} // texture painting handled separately
                }
            }
        }
        self.dirty = true;
    }

    /// Get the average height of neighbors at a grid position.
    #[must_use]
    fn neighbor_average(&self, x: u32, z: u32) -> f32 {
        let cols = self.width + 1;
        let rows = self.depth + 1;
        let mut sum = 0.0;
        let mut count = 0;

        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let nx = x as i32 + dx;
                let nz = z as i32 + dz;
                if nx >= 0 && nx < cols as i32 && nz >= 0 && nz < rows as i32 {
                    let idx = (nz as u32 * cols + nx as u32) as usize;
                    sum += self.heights[idx];
                    count += 1;
                }
            }
        }
        if count > 0 { sum / count as f32 } else { 0.0 }
    }

    /// Build a soorat TerrainConfig from the editor state.
    #[must_use]
    pub fn to_terrain_config(&self) -> soorat::terrain::TerrainConfig {
        soorat::terrain::TerrainConfig {
            width: self.width,
            depth: self.depth,
            world_width: self.width as f32,
            world_depth: self.depth as f32,
            height_scale: self.height_scale,
        }
    }

    /// Reset the heightmap to flat.
    pub fn reset(&mut self) {
        self.heights.fill(0.0);
        self.dirty = true;
    }

    /// Get the height at a specific grid position.
    #[must_use]
    pub fn height_at(&self, x: u32, z: u32) -> f32 {
        let cols = self.width + 1;
        let idx = (z * cols + x) as usize;
        self.heights.get(idx).copied().unwrap_or(0.0)
    }

    /// Min and max heights in the current heightmap.
    #[must_use]
    pub fn height_range(&self) -> (f32, f32) {
        let min = self
            .heights
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max = self
            .heights
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        (min, max)
    }
}

/// Render the terrain editing panel.
pub fn terrain_panel(ui: &mut egui::Ui, editor: &mut TerrainEditor) {
    ui.heading("Terrain");
    ui.separator();

    // Brush tool selector
    ui.horizontal(|ui| {
        ui.label("Brush:");
        for tool in [
            BrushTool::Raise,
            BrushTool::Lower,
            BrushTool::Smooth,
            BrushTool::Flatten,
        ] {
            if ui
                .selectable_label(editor.brush == tool, tool.label())
                .clicked()
            {
                editor.brush = tool;
            }
        }
    });

    // Brush settings
    egui::Grid::new("terrain_brush_settings").show(ui, |ui| {
        ui.label("Radius:");
        ui.add(egui::Slider::new(&mut editor.brush_radius, 1.0..=20.0));
        ui.end_row();

        ui.label("Strength:");
        ui.add(egui::Slider::new(&mut editor.brush_strength, 0.01..=1.0));
        ui.end_row();

        if editor.brush == BrushTool::Flatten {
            ui.label("Target:");
            ui.add(egui::DragValue::new(&mut editor.flatten_height).speed(0.1));
            ui.end_row();
        }

        ui.label("Height Scale:");
        ui.add(
            egui::DragValue::new(&mut editor.height_scale)
                .speed(0.5)
                .range(1.0..=100.0),
        );
        ui.end_row();
    });

    ui.separator();

    // Terrain info
    let (min_h, max_h) = editor.height_range();
    ui.label(format!(
        "Grid: {}x{} | Height range: {:.2} to {:.2}",
        editor.width, editor.depth, min_h, max_h
    ));

    if editor.dirty {
        ui.colored_label(egui::Color32::from_rgb(220, 180, 60), "Modified");
    }

    ui.horizontal(|ui| {
        if ui.button("Reset Flat").clicked() {
            editor.reset();
            tracing::info!("terrain reset to flat");
        }
    });

    ui.separator();

    // Heightmap preview (top-down grayscale)
    let preview_size = ui.available_width().min(200.0);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(preview_size, preview_size), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(20, 20, 25));

    let (h_min, h_max) = editor.height_range();
    let h_range = (h_max - h_min).max(0.001);
    let cols = editor.width + 1;
    let rows = editor.depth + 1;
    let cell_w = rect.width() / cols as f32;
    let cell_h = rect.height() / rows as f32;

    // Draw a low-res preview (skip cells if terrain is large)
    let step = ((cols.max(rows)) / 64).max(1);
    for z in (0..rows).step_by(step as usize) {
        for x in (0..cols).step_by(step as usize) {
            let h = editor.height_at(x, z);
            let t = ((h - h_min) / h_range).clamp(0.0, 1.0);
            let gray = (t * 255.0) as u8;
            let px = rect.left() + x as f32 * cell_w;
            let py = rect.top() + z as f32 * cell_h;
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(px, py),
                    egui::vec2(cell_w * step as f32, cell_h * step as f32),
                ),
                0.0,
                egui::Color32::from_rgb(gray, gray, gray),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_default() {
        let e = TerrainEditor::default();
        assert_eq!(e.width, 64);
        assert_eq!(e.depth, 64);
        assert_eq!(e.heights.len(), 65 * 65);
        assert!(!e.dirty);
    }

    #[test]
    fn editor_new_custom_size() {
        let e = TerrainEditor::new(32, 16);
        assert_eq!(e.width, 32);
        assert_eq!(e.depth, 16);
        assert_eq!(e.heights.len(), 33 * 17);
    }

    #[test]
    fn brush_raise() {
        let mut e = TerrainEditor::new(10, 10);
        e.brush = BrushTool::Raise;
        e.brush_strength = 1.0;
        e.brush_radius = 1.0;
        e.apply_brush(5.0, 5.0);
        assert!(e.heights[(5 * 11 + 5) as usize] > 0.0);
        assert!(e.dirty);
    }

    #[test]
    fn brush_lower() {
        let mut e = TerrainEditor::new(10, 10);
        e.heights.fill(1.0);
        e.brush = BrushTool::Lower;
        e.brush_strength = 1.0;
        e.brush_radius = 1.0;
        e.apply_brush(5.0, 5.0);
        assert!(e.heights[(5 * 11 + 5) as usize] < 1.0);
    }

    #[test]
    fn brush_flatten() {
        let mut e = TerrainEditor::new(10, 10);
        e.heights.fill(5.0);
        e.brush = BrushTool::Flatten;
        e.flatten_height = 0.0;
        e.brush_strength = 1.0;
        e.brush_radius = 2.0;
        e.apply_brush(5.0, 5.0);
        assert!(e.heights[(5 * 11 + 5) as usize] < 5.0);
    }

    #[test]
    fn brush_smooth() {
        let mut e = TerrainEditor::new(10, 10);
        e.heights[(5 * 11 + 5) as usize] = 10.0; // spike
        e.brush = BrushTool::Smooth;
        e.brush_strength = 1.0;
        e.brush_radius = 2.0;
        e.apply_brush(5.0, 5.0);
        assert!(e.heights[(5 * 11 + 5) as usize] < 10.0);
    }

    #[test]
    fn reset_clears_heightmap() {
        let mut e = TerrainEditor::new(10, 10);
        e.heights.fill(5.0);
        e.reset();
        assert!(e.heights.iter().all(|&h| h == 0.0));
        assert!(e.dirty);
    }

    #[test]
    fn height_at_valid() {
        let mut e = TerrainEditor::new(10, 10);
        e.heights[(3 * 11 + 4) as usize] = 7.5;
        assert_eq!(e.height_at(4, 3), 7.5);
    }

    #[test]
    fn height_at_out_of_bounds() {
        let e = TerrainEditor::new(10, 10);
        assert_eq!(e.height_at(999, 999), 0.0);
    }

    #[test]
    fn height_range_flat() {
        let e = TerrainEditor::new(10, 10);
        let (min, max) = e.height_range();
        assert_eq!(min, 0.0);
        assert_eq!(max, 0.0);
    }

    #[test]
    fn height_range_varied() {
        let mut e = TerrainEditor::new(10, 10);
        e.heights[0] = -5.0;
        e.heights[1] = 10.0;
        let (min, max) = e.height_range();
        assert_eq!(min, -5.0);
        assert_eq!(max, 10.0);
    }

    #[test]
    fn to_terrain_config() {
        let e = TerrainEditor::new(32, 16);
        let config = e.to_terrain_config();
        assert_eq!(config.width, 32);
        assert_eq!(config.depth, 16);
        assert_eq!(config.height_scale, 10.0);
    }

    #[test]
    fn brush_tool_labels() {
        for tool in [
            BrushTool::Raise,
            BrushTool::Lower,
            BrushTool::Smooth,
            BrushTool::Flatten,
            BrushTool::Paint,
        ] {
            assert!(!tool.label().is_empty());
        }
    }

    #[test]
    fn brush_falloff() {
        let mut e = TerrainEditor::new(10, 10);
        e.brush = BrushTool::Raise;
        e.brush_strength = 1.0;
        e.brush_radius = 3.0;
        e.apply_brush(5.0, 5.0);
        let center = e.heights[(5 * 11 + 5) as usize];
        let edge = e.heights[(5 * 11 + 7) as usize];
        // Center should be raised more than edge
        assert!(center > edge);
    }
}
