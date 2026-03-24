//! Console/log panel — displays captured tracing events with severity filtering.

use std::collections::VecDeque;

/// Severity level for console entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    #[must_use]
    pub fn color(self) -> egui::Color32 {
        match self {
            LogLevel::Debug => egui::Color32::from_rgb(140, 140, 140),
            LogLevel::Info => egui::Color32::from_rgb(180, 200, 220),
            LogLevel::Warn => egui::Color32::from_rgb(220, 200, 80),
            LogLevel::Error => egui::Color32::from_rgb(220, 80, 80),
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Debug => "DBG",
            LogLevel::Info => "INF",
            LogLevel::Warn => "WRN",
            LogLevel::Error => "ERR",
        }
    }
}

/// A single log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Severity level.
    pub level: LogLevel,
    /// Source module/system.
    pub source: String,
    /// Log message.
    pub message: String,
    /// Frame number when logged.
    pub frame: u64,
}

/// Console state — ring buffer of log entries.
pub struct Console {
    /// Log entries (newest at back).
    entries: VecDeque<LogEntry>,
    /// Maximum entries to keep.
    max_entries: usize,
    /// Minimum severity to display.
    pub min_level: LogLevel,
    /// Auto-scroll to bottom.
    pub auto_scroll: bool,
    /// Frame counter for log entries.
    frame: u64,
}

impl Default for Console {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl Console {
    /// Create a console with the given capacity.
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
            min_level: LogLevel::Info,
            auto_scroll: true,
            frame: 0,
        }
    }

    /// Push a log entry. Evicts oldest if at capacity.
    pub fn log(&mut self, level: LogLevel, source: impl Into<String>, message: impl Into<String>) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(LogEntry {
            level,
            source: source.into(),
            message: message.into(),
            frame: self.frame,
        });
    }

    /// Convenience methods.
    pub fn info(&mut self, source: &str, message: impl Into<String>) {
        self.log(LogLevel::Info, source, message);
    }

    pub fn warn(&mut self, source: &str, message: impl Into<String>) {
        self.log(LogLevel::Warn, source, message);
    }

    pub fn error(&mut self, source: &str, message: impl Into<String>) {
        self.log(LogLevel::Error, source, message);
    }

    pub fn debug(&mut self, source: &str, message: impl Into<String>) {
        self.log(LogLevel::Debug, source, message);
    }

    /// Advance the frame counter.
    pub fn tick(&mut self) {
        self.frame += 1;
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get filtered entries.
    #[must_use]
    pub fn filtered_entries(&self) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.level >= self.min_level)
            .collect()
    }

    /// Total entry count.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the console is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Render the console panel.
pub fn console_panel(ui: &mut egui::Ui, console: &mut Console) {
    ui.heading("Console");

    // Controls
    ui.horizontal(|ui| {
        if ui.button("Clear").clicked() {
            console.clear();
        }
        ui.separator();
        ui.label("Filter:");
        for level in [
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            if ui
                .selectable_label(console.min_level == level, level.label())
                .clicked()
            {
                console.min_level = level;
            }
        }
        ui.separator();
        ui.checkbox(&mut console.auto_scroll, "Auto-scroll");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("{} entries", console.len()));
        });
    });
    ui.separator();

    // Log entries
    let filtered = console.filtered_entries();
    let scroll = egui::ScrollArea::vertical().stick_to_bottom(console.auto_scroll);
    scroll.show(ui, |ui| {
        for entry in &filtered {
            ui.horizontal(|ui| {
                ui.colored_label(entry.level.color(), entry.level.label());
                ui.label(
                    egui::RichText::new(format!("[{}]", entry.source))
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.label(&entry.message);
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_default() {
        let c = Console::default();
        assert!(c.is_empty());
        assert_eq!(c.min_level, LogLevel::Info);
        assert!(c.auto_scroll);
    }

    #[test]
    fn console_log_and_count() {
        let mut c = Console::new(100);
        c.info("test", "hello");
        c.warn("test", "warning");
        c.error("test", "error");
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn console_eviction() {
        let mut c = Console::new(3);
        c.info("a", "1");
        c.info("b", "2");
        c.info("c", "3");
        c.info("d", "4");
        assert_eq!(c.len(), 3);
        // Oldest entry evicted
        let entries: Vec<_> = c.entries.iter().map(|e| e.source.as_str()).collect();
        assert_eq!(entries, vec!["b", "c", "d"]);
    }

    #[test]
    fn console_filter() {
        let mut c = Console::new(100);
        c.debug("d", "debug msg");
        c.info("i", "info msg");
        c.warn("w", "warn msg");
        c.error("e", "error msg");

        c.min_level = LogLevel::Warn;
        let filtered = c.filtered_entries();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].level, LogLevel::Warn);
        assert_eq!(filtered[1].level, LogLevel::Error);
    }

    #[test]
    fn console_clear() {
        let mut c = Console::new(100);
        c.info("t", "msg");
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn console_tick() {
        let mut c = Console::new(100);
        c.tick();
        c.info("t", "after tick");
        assert_eq!(c.entries.back().unwrap().frame, 1);
    }

    #[test]
    fn log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }
}
