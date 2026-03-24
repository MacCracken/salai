//! Asset browser panel — scan directories and preview textures/audio/scenes.

use std::path::{Path, PathBuf};

/// Recognized asset types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssetKind {
    Texture,
    Audio,
    Scene,
    Unknown,
}

impl AssetKind {
    /// Classify a file by extension.
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "tga" | "webp" | "tiff" => AssetKind::Texture,
            "wav" | "mp3" | "ogg" | "flac" | "aiff" => AssetKind::Audio,
            "toml" => AssetKind::Scene,
            _ => AssetKind::Unknown,
        }
    }

    /// Icon/label for display.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            AssetKind::Texture => "IMG",
            AssetKind::Audio => "SND",
            AssetKind::Scene => "SCN",
            AssetKind::Unknown => "???",
        }
    }

    /// Color for the asset kind badge.
    #[must_use]
    pub fn color(self) -> egui::Color32 {
        match self {
            AssetKind::Texture => egui::Color32::from_rgb(100, 180, 100),
            AssetKind::Audio => egui::Color32::from_rgb(100, 140, 220),
            AssetKind::Scene => egui::Color32::from_rgb(220, 180, 80),
            AssetKind::Unknown => egui::Color32::GRAY,
        }
    }
}

/// A single asset entry in the browser.
#[derive(Debug, Clone)]
pub struct AssetEntry {
    /// File name (without path).
    pub name: String,
    /// Full path.
    pub path: PathBuf,
    /// Asset type.
    pub kind: AssetKind,
    /// File size in bytes.
    pub size: u64,
}

/// State for the asset browser.
pub struct AssetBrowser {
    /// Root directory being browsed.
    pub root: Option<PathBuf>,
    /// Scanned asset entries.
    pub entries: Vec<AssetEntry>,
    /// Currently selected asset index.
    pub selected: Option<usize>,
    /// Filter by asset kind (None = show all).
    pub filter: Option<AssetKind>,
}

impl Default for AssetBrowser {
    fn default() -> Self {
        Self {
            root: None,
            entries: Vec::new(),
            selected: None,
            filter: None,
        }
    }
}

impl AssetBrowser {
    /// Scan a directory for assets.
    pub fn scan(&mut self, dir: &Path) {
        self.root = Some(dir.to_path_buf());
        self.entries.clear();
        self.selected = None;

        if let Ok(read_dir) = std::fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let kind = AssetKind::from_extension(ext);
                if kind == AssetKind::Unknown {
                    continue;
                }
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();

                self.entries.push(AssetEntry {
                    name,
                    path,
                    kind,
                    size,
                });
            }
        }

        self.entries.sort_by(|a, b| a.name.cmp(&b.name));
        tracing::info!(
            dir = %dir.display(),
            count = self.entries.len(),
            "asset directory scanned"
        );
    }

    /// Get filtered entries.
    #[must_use]
    pub fn filtered_entries(&self) -> Vec<&AssetEntry> {
        match self.filter {
            Some(kind) => self.entries.iter().filter(|e| e.kind == kind).collect(),
            None => self.entries.iter().collect(),
        }
    }

    /// Get the currently selected asset.
    #[must_use]
    pub fn selected_asset(&self) -> Option<&AssetEntry> {
        self.selected.and_then(|i| self.entries.get(i))
    }
}

/// Format file size for display.
#[must_use]
pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Render the asset browser panel.
pub fn asset_browser_panel(ui: &mut egui::Ui, browser: &mut AssetBrowser) {
    ui.heading("Assets");
    ui.separator();

    // Filter buttons
    ui.horizontal(|ui| {
        if ui
            .selectable_label(browser.filter.is_none(), "All")
            .clicked()
        {
            browser.filter = None;
        }
        for kind in [AssetKind::Texture, AssetKind::Audio, AssetKind::Scene] {
            if ui
                .selectable_label(browser.filter == Some(kind), kind.label())
                .clicked()
            {
                browser.filter = Some(kind);
            }
        }
    });
    ui.separator();

    if browser.entries.is_empty() {
        ui.label("No assets loaded. Use File > Open to set asset directory.");
        return;
    }

    // Pre-compute filtered indices to avoid borrow conflict
    let filtered_indices: Vec<usize> = browser
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| browser.filter.is_none() || browser.filter == Some(e.kind))
        .map(|(i, _)| i)
        .collect();

    ui.label(format!("{} assets", filtered_indices.len()));

    egui::ScrollArea::vertical().show(ui, |ui| {
        for &idx in &filtered_indices {
            let entry = &browser.entries[idx];
            let selected = browser.selected == Some(idx);
            let name = entry.name.clone();
            let kind = entry.kind;
            let size = entry.size;

            ui.horizontal(|ui| {
                ui.colored_label(kind.color(), kind.label());
                if ui.selectable_label(selected, &name).clicked() {
                    browser.selected = Some(idx);
                    tracing::debug!(asset = %name, kind = ?kind, "asset selected");
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format_size(size))
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                });
            });
        }
    });

    // Preview area for selected asset
    if let Some(asset) = browser.selected_asset() {
        ui.separator();
        ui.label(format!("Selected: {}", asset.name));
        ui.label(format!(
            "Type: {:?}  Size: {}",
            asset.kind,
            format_size(asset.size)
        ));
        ui.label(format!("Path: {}", asset.path.display()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_kind_from_extension() {
        assert_eq!(AssetKind::from_extension("png"), AssetKind::Texture);
        assert_eq!(AssetKind::from_extension("JPG"), AssetKind::Texture);
        assert_eq!(AssetKind::from_extension("wav"), AssetKind::Audio);
        assert_eq!(AssetKind::from_extension("toml"), AssetKind::Scene);
        assert_eq!(AssetKind::from_extension("xyz"), AssetKind::Unknown);
    }

    #[test]
    fn asset_kind_label() {
        assert_eq!(AssetKind::Texture.label(), "IMG");
        assert_eq!(AssetKind::Audio.label(), "SND");
        assert_eq!(AssetKind::Scene.label(), "SCN");
    }

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1_500_000), "1.4 MB");
    }

    #[test]
    fn browser_default_empty() {
        let browser = AssetBrowser::default();
        assert!(browser.entries.is_empty());
        assert!(browser.selected.is_none());
        assert!(browser.filter.is_none());
    }

    #[test]
    fn browser_scan_nonexistent() {
        let mut browser = AssetBrowser::default();
        browser.scan(Path::new("/nonexistent/dir"));
        assert!(browser.entries.is_empty());
    }

    #[test]
    fn browser_filter() {
        let mut browser = AssetBrowser::default();
        browser.entries.push(AssetEntry {
            name: "tex.png".into(),
            path: PathBuf::from("tex.png"),
            kind: AssetKind::Texture,
            size: 1000,
        });
        browser.entries.push(AssetEntry {
            name: "snd.wav".into(),
            path: PathBuf::from("snd.wav"),
            kind: AssetKind::Audio,
            size: 2000,
        });

        assert_eq!(browser.filtered_entries().len(), 2);

        browser.filter = Some(AssetKind::Texture);
        assert_eq!(browser.filtered_entries().len(), 1);
        assert_eq!(browser.filtered_entries()[0].name, "tex.png");
    }

    #[test]
    fn browser_selected_asset() {
        let mut browser = AssetBrowser::default();
        browser.entries.push(AssetEntry {
            name: "level.toml".into(),
            path: PathBuf::from("level.toml"),
            kind: AssetKind::Scene,
            size: 500,
        });
        assert!(browser.selected_asset().is_none());

        browser.selected = Some(0);
        let asset = browser.selected_asset().unwrap();
        assert_eq!(asset.name, "level.toml");
    }
}
