//! Hardware capability detection for adaptive viewport configuration.
//!
//! Uses [`ai_hwaccel`] to probe the system for GPU/accelerator hardware and
//! maps the results to editor-relevant quality tiers and feature flags.

use ai_hwaccel::{AcceleratorFamily, AcceleratorProfile, AcceleratorRegistry};

/// Quality tier for viewport rendering, derived from hardware capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QualityTier {
    /// Software fallback — CPU only, minimal effects.
    Low,
    /// Integrated or low-end discrete GPU.
    #[default]
    Medium,
    /// Discrete GPU with adequate VRAM (4+ GiB).
    High,
    /// High-end discrete GPU (8+ GiB VRAM, high bandwidth).
    Ultra,
}

impl std::fmt::Display for QualityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityTier::Low => write!(f, "Low"),
            QualityTier::Medium => write!(f, "Medium"),
            QualityTier::High => write!(f, "High"),
            QualityTier::Ultra => write!(f, "Ultra"),
        }
    }
}

/// Editor-relevant hardware profile derived from system detection.
#[derive(Debug, Clone)]
pub struct HardwareProfile {
    /// Recommended quality tier for the viewport.
    pub quality: QualityTier,
    /// Whether a discrete GPU (or dedicated accelerator) is available.
    pub has_gpu: bool,
    /// Total GPU/accelerator memory in bytes (0 if CPU-only).
    pub gpu_memory_bytes: u64,
    /// Name of the best available device.
    pub device_name: String,
    /// Number of detected accelerators (excluding CPU).
    pub accelerator_count: usize,
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            quality: QualityTier::Medium,
            has_gpu: false,
            gpu_memory_bytes: 0,
            device_name: "Unknown".into(),
            accelerator_count: 0,
        }
    }
}

impl HardwareProfile {
    /// Detect hardware and build a profile for the editor.
    pub fn detect() -> Self {
        let registry = AcceleratorRegistry::detect();
        Self::from_registry(&registry)
    }

    /// Build a profile from an existing registry (useful for testing).
    pub fn from_registry(registry: &AcceleratorRegistry) -> Self {
        let has_gpu = registry.has_accelerator();
        let gpu_memory_bytes = registry.total_accelerator_memory();
        let accelerator_count = registry
            .available()
            .iter()
            .filter(|p| !matches!(p.accelerator.family(), AcceleratorFamily::Cpu))
            .count();

        let device_name = registry
            .best_available()
            .map(|p| format!("{}", p.accelerator))
            .unwrap_or_else(|| "CPU".into());

        let quality = classify_quality(registry.best_available(), gpu_memory_bytes);

        Self {
            quality,
            has_gpu,
            gpu_memory_bytes,
            device_name,
            accelerator_count,
        }
    }

    /// Suggested grid size based on quality tier.
    pub fn suggested_grid_size(&self) -> f32 {
        match self.quality {
            QualityTier::Low => 2.0,
            QualityTier::Medium => 1.0,
            QualityTier::High => 0.5,
            QualityTier::Ultra => 0.25,
        }
    }

    /// Whether debug shapes should be enabled by default.
    pub fn default_debug_shapes(&self) -> bool {
        // Only enable by default on High/Ultra where the GPU can handle it
        matches!(self.quality, QualityTier::High | QualityTier::Ultra)
    }

    /// GPU memory in human-readable format.
    pub fn gpu_memory_display(&self) -> String {
        if self.gpu_memory_bytes == 0 {
            return "N/A".into();
        }
        let gib = self.gpu_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if gib >= 1.0 {
            format!("{gib:.1} GiB")
        } else {
            let mib = self.gpu_memory_bytes as f64 / (1024.0 * 1024.0);
            format!("{mib:.0} MiB")
        }
    }
}

/// Map hardware capabilities to a quality tier.
fn classify_quality(best: Option<&AcceleratorProfile>, total_vram: u64) -> QualityTier {
    let Some(profile) = best else {
        return QualityTier::Low;
    };

    if matches!(profile.accelerator.family(), AcceleratorFamily::Cpu) {
        return QualityTier::Low;
    }

    let gib = total_vram / (1024 * 1024 * 1024);

    match gib {
        0..=1 => QualityTier::Medium,
        2..=3 => QualityTier::Medium,
        4..=7 => QualityTier::High,
        _ => QualityTier::Ultra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile() {
        let p = HardwareProfile::default();
        assert_eq!(p.quality, QualityTier::Medium);
        assert!(!p.has_gpu);
        assert_eq!(p.gpu_memory_bytes, 0);
        assert_eq!(p.device_name, "Unknown");
    }

    #[test]
    fn quality_tier_display() {
        assert_eq!(QualityTier::Low.to_string(), "Low");
        assert_eq!(QualityTier::Medium.to_string(), "Medium");
        assert_eq!(QualityTier::High.to_string(), "High");
        assert_eq!(QualityTier::Ultra.to_string(), "Ultra");
    }

    #[test]
    fn classify_quality_no_profile() {
        assert_eq!(classify_quality(None, 0), QualityTier::Low);
    }

    #[test]
    fn classify_quality_cpu_only() {
        let cpu = AcceleratorProfile::cpu(16 * 1024 * 1024 * 1024);
        assert_eq!(classify_quality(Some(&cpu), 0), QualityTier::Low);
    }

    #[test]
    fn classify_quality_small_gpu() {
        let gpu = AcceleratorProfile::cuda(0, 2 * 1024 * 1024 * 1024);
        assert_eq!(
            classify_quality(Some(&gpu), 2 * 1024 * 1024 * 1024),
            QualityTier::Medium
        );
    }

    #[test]
    fn classify_quality_mid_gpu() {
        let gpu = AcceleratorProfile::cuda(0, 6 * 1024 * 1024 * 1024);
        assert_eq!(
            classify_quality(Some(&gpu), 6 * 1024 * 1024 * 1024),
            QualityTier::High
        );
    }

    #[test]
    fn classify_quality_high_gpu() {
        let gpu = AcceleratorProfile::cuda(0, 12 * 1024 * 1024 * 1024);
        assert_eq!(
            classify_quality(Some(&gpu), 12 * 1024 * 1024 * 1024),
            QualityTier::Ultra
        );
    }

    #[test]
    fn suggested_grid_size_varies() {
        let mut p = HardwareProfile::default();
        p.quality = QualityTier::Low;
        assert_eq!(p.suggested_grid_size(), 2.0);
        p.quality = QualityTier::Ultra;
        assert_eq!(p.suggested_grid_size(), 0.25);
    }

    #[test]
    fn default_debug_shapes_varies() {
        let mut p = HardwareProfile::default();
        p.quality = QualityTier::Low;
        assert!(!p.default_debug_shapes());
        p.quality = QualityTier::High;
        assert!(p.default_debug_shapes());
    }

    #[test]
    fn gpu_memory_display_zero() {
        let p = HardwareProfile::default();
        assert_eq!(p.gpu_memory_display(), "N/A");
    }

    #[test]
    fn gpu_memory_display_gib() {
        let mut p = HardwareProfile::default();
        p.gpu_memory_bytes = 8 * 1024 * 1024 * 1024;
        assert_eq!(p.gpu_memory_display(), "8.0 GiB");
    }

    #[test]
    fn gpu_memory_display_mib() {
        let mut p = HardwareProfile::default();
        p.gpu_memory_bytes = 512 * 1024 * 1024;
        assert_eq!(p.gpu_memory_display(), "512 MiB");
    }

    #[test]
    fn detect_returns_valid_profile() {
        // Actually runs detection — should always succeed (at least CPU)
        let p = HardwareProfile::detect();
        assert!(!p.device_name.is_empty());
    }

    #[test]
    fn from_registry_cpu_fallback() {
        let registry = AcceleratorRegistry::detect();
        let p = HardwareProfile::from_registry(&registry);
        // Should always produce a valid profile
        assert!(!p.device_name.is_empty());
    }
}
