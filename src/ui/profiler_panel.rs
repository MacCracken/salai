//! Performance profiler panel — frame times, entity stats, memory estimates.

use std::collections::VecDeque;

/// Profiler state — tracks frame timing and scene statistics.
pub struct Profiler {
    /// Recent frame times in seconds.
    frame_times: VecDeque<f64>,
    /// Maximum frame history.
    max_history: usize,
    /// Current frame count.
    pub frame_count: u64,
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new(120)
    }
}

impl Profiler {
    /// Create a profiler with the given frame history depth.
    #[must_use]
    pub fn new(max_history: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_history),
            max_history,
            frame_count: 0,
        }
    }

    /// Record a frame time (in seconds).
    pub fn record_frame(&mut self, dt: f64) {
        if self.frame_times.len() >= self.max_history {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(dt);
        self.frame_count += 1;
    }

    /// Average frame time over the history window.
    #[must_use]
    pub fn avg_frame_time(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64
    }

    /// Estimated FPS from average frame time.
    #[must_use]
    pub fn fps(&self) -> f64 {
        let avg = self.avg_frame_time();
        if avg > 0.0 { 1.0 / avg } else { 0.0 }
    }

    /// Maximum frame time in the history (worst frame).
    #[must_use]
    pub fn max_frame_time(&self) -> f64 {
        self.frame_times
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Minimum frame time in the history (best frame).
    #[must_use]
    pub fn min_frame_time(&self) -> f64 {
        self.frame_times
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Get frame times for graphing.
    #[must_use]
    pub fn frame_times(&self) -> &VecDeque<f64> {
        &self.frame_times
    }

    /// Clear history.
    pub fn clear(&mut self) {
        self.frame_times.clear();
        self.frame_count = 0;
    }
}

/// Render the profiler panel.
pub fn profiler_panel(
    ui: &mut egui::Ui,
    profiler: &Profiler,
    entity_count: usize,
    history_size: usize,
) {
    ui.heading("Profiler");
    ui.separator();

    // FPS and frame time
    ui.horizontal(|ui| {
        let fps = profiler.fps();
        let color = if fps >= 55.0 {
            egui::Color32::from_rgb(100, 200, 100)
        } else if fps >= 30.0 {
            egui::Color32::from_rgb(220, 200, 80)
        } else {
            egui::Color32::from_rgb(220, 80, 80)
        };
        ui.colored_label(color, format!("{fps:.0} FPS"));
        ui.separator();
        ui.label(format!("avg: {:.1} ms", profiler.avg_frame_time() * 1000.0));
        ui.label(format!("max: {:.1} ms", profiler.max_frame_time() * 1000.0));
    });

    ui.separator();

    // Frame time graph
    let times = profiler.frame_times();
    if !times.is_empty() {
        let max_dt = profiler.max_frame_time().max(0.020); // at least 20ms scale
        let graph_height = 60.0;
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), graph_height),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(20, 20, 25));

        // 16ms target line (60 FPS)
        let target_y = rect.bottom() - (0.01667 / max_dt) as f32 * rect.height();
        painter.line_segment(
            [
                egui::pos2(rect.left(), target_y),
                egui::pos2(rect.right(), target_y),
            ],
            egui::Stroke::new(0.5, egui::Color32::from_rgb(60, 100, 60)),
        );

        // Frame time bars
        let bar_width = rect.width() / times.len().max(1) as f32;
        for (i, &dt) in times.iter().enumerate() {
            let height = (dt / max_dt) as f32 * rect.height();
            let x = rect.left() + i as f32 * bar_width;
            let bar_rect = egui::Rect::from_min_size(
                egui::pos2(x, rect.bottom() - height),
                egui::vec2(bar_width.max(1.0), height),
            );
            let color = if dt < 0.01667 {
                egui::Color32::from_rgb(80, 160, 80)
            } else if dt < 0.033 {
                egui::Color32::from_rgb(200, 180, 60)
            } else {
                egui::Color32::from_rgb(200, 60, 60)
            };
            painter.rect_filled(bar_rect, 0.0, color);
        }
    }

    ui.separator();

    // Scene stats
    egui::Grid::new("profiler_stats").show(ui, |ui| {
        ui.label("Entities:");
        ui.label(format!("{entity_count}"));
        ui.end_row();

        ui.label("History:");
        ui.label(format!("{history_size} actions"));
        ui.end_row();

        ui.label("Frames:");
        ui.label(format!("{}", profiler.frame_count));
        ui.end_row();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiler_default() {
        let p = Profiler::default();
        assert_eq!(p.frame_count, 0);
        assert_eq!(p.fps(), 0.0);
        assert_eq!(p.avg_frame_time(), 0.0);
    }

    #[test]
    fn profiler_record_and_stats() {
        let mut p = Profiler::new(10);
        for _ in 0..5 {
            p.record_frame(0.016); // ~60 FPS
        }
        assert_eq!(p.frame_count, 5);
        assert!((p.avg_frame_time() - 0.016).abs() < 1e-10);
        assert!((p.fps() - 62.5).abs() < 1.0);
    }

    #[test]
    fn profiler_eviction() {
        let mut p = Profiler::new(3);
        p.record_frame(0.010);
        p.record_frame(0.020);
        p.record_frame(0.030);
        p.record_frame(0.040);
        assert_eq!(p.frame_times().len(), 3);
        assert_eq!(*p.frame_times().front().unwrap(), 0.020);
    }

    #[test]
    fn profiler_min_max() {
        let mut p = Profiler::new(10);
        p.record_frame(0.010);
        p.record_frame(0.050);
        p.record_frame(0.020);
        assert!((p.min_frame_time() - 0.010).abs() < 1e-10);
        assert!((p.max_frame_time() - 0.050).abs() < 1e-10);
    }

    #[test]
    fn profiler_clear() {
        let mut p = Profiler::new(10);
        p.record_frame(0.016);
        p.clear();
        assert_eq!(p.frame_count, 0);
        assert!(p.frame_times().is_empty());
    }
}
