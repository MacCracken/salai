//! NPC personality editing panel — trait sliders, mood display, compatibility.

use crate::personality::{ALL_LEVELS, NpcPersonality};
use bhava::traits::{TraitGroup, TraitKind, TraitLevel};

/// Group metadata for display.
struct TraitGroupInfo {
    group: TraitGroup,
    label: &'static str,
    traits: &'static [TraitKind],
}

const GROUPS: &[TraitGroupInfo] = &[
    TraitGroupInfo {
        group: TraitGroup::Social,
        label: "Social",
        traits: &[
            TraitKind::Warmth,
            TraitKind::Empathy,
            TraitKind::Humor,
            TraitKind::Patience,
        ],
    },
    TraitGroupInfo {
        group: TraitGroup::Cognitive,
        label: "Cognitive",
        traits: &[
            TraitKind::Curiosity,
            TraitKind::Creativity,
            TraitKind::Confidence,
            TraitKind::Skepticism,
        ],
    },
    TraitGroupInfo {
        group: TraitGroup::Behavioral,
        label: "Behavioral",
        traits: &[
            TraitKind::Formality,
            TraitKind::Verbosity,
            TraitKind::Directness,
            TraitKind::RiskTolerance,
        ],
    },
    TraitGroupInfo {
        group: TraitGroup::Professional,
        label: "Professional",
        traits: &[
            TraitKind::Autonomy,
            TraitKind::Pedagogy,
            TraitKind::Precision,
        ],
    },
];

/// Convert TraitLevel to slider index (0-4).
#[must_use]
fn level_to_index(level: TraitLevel) -> usize {
    match level {
        TraitLevel::Lowest => 0,
        TraitLevel::Low => 1,
        TraitLevel::Balanced => 2,
        TraitLevel::High => 3,
        TraitLevel::Highest => 4,
    }
}

/// Convert slider index to TraitLevel.
#[must_use]
fn index_to_level(index: usize) -> TraitLevel {
    ALL_LEVELS[index.min(4)]
}

/// Label for a TraitLevel.
#[must_use]
fn level_label(level: TraitLevel) -> &'static str {
    match level {
        TraitLevel::Lowest => "Lowest",
        TraitLevel::Low => "Low",
        TraitLevel::Balanced => "Balanced",
        TraitLevel::High => "High",
        TraitLevel::Highest => "Highest",
    }
}

/// Color for a TraitLevel.
#[must_use]
fn level_color(level: TraitLevel) -> egui::Color32 {
    match level {
        TraitLevel::Lowest => egui::Color32::from_rgb(180, 60, 60),
        TraitLevel::Low => egui::Color32::from_rgb(200, 140, 80),
        TraitLevel::Balanced => egui::Color32::from_rgb(150, 150, 150),
        TraitLevel::High => egui::Color32::from_rgb(80, 160, 200),
        TraitLevel::Highest => egui::Color32::from_rgb(60, 180, 100),
    }
}

/// Display name for a TraitKind.
#[must_use]
fn trait_label(kind: TraitKind) -> &'static str {
    match kind {
        TraitKind::Formality => "Formality",
        TraitKind::Humor => "Humor",
        TraitKind::Verbosity => "Verbosity",
        TraitKind::Directness => "Directness",
        TraitKind::Warmth => "Warmth",
        TraitKind::Empathy => "Empathy",
        TraitKind::Patience => "Patience",
        TraitKind::Confidence => "Confidence",
        TraitKind::Creativity => "Creativity",
        TraitKind::RiskTolerance => "Risk Tolerance",
        TraitKind::Curiosity => "Curiosity",
        TraitKind::Skepticism => "Skepticism",
        TraitKind::Autonomy => "Autonomy",
        TraitKind::Pedagogy => "Pedagogy",
        TraitKind::Precision => "Precision",
        _ => "Unknown",
    }
}

/// Render the full personality editing panel.
pub fn personality_panel(ui: &mut egui::Ui, npc: &mut NpcPersonality) {
    ui.heading("Personality");
    ui.separator();

    // Summary bar
    let summary = npc.inspector_summary();
    ui.horizontal(|ui| {
        ui.label(format!("Name: {}", summary.name));
        ui.separator();
        ui.label(format!("{} active traits", summary.trait_count));
        ui.separator();
        ui.colored_label(egui::Color32::from_rgb(180, 180, 220), &summary.mood_label);
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Trait groups
        for group_info in GROUPS {
            trait_group_section(ui, npc, group_info);
        }

        ui.separator();

        // Mood display
        mood_section(ui, &npc.mood);
    });
}

/// Render a trait group with sliders.
fn trait_group_section(ui: &mut egui::Ui, npc: &mut NpcPersonality, group: &TraitGroupInfo) {
    let avg = npc.group_average(group.group);
    let avg_label = if avg.abs() < 0.1 {
        "neutral".to_string()
    } else {
        format!("{avg:+.1}")
    };

    egui::CollapsingHeader::new(format!("{} ({})", group.label, avg_label))
        .default_open(true)
        .show(ui, |ui| {
            for &kind in group.traits {
                trait_slider(ui, npc, kind);
            }
        });
}

/// Render a single trait as a labeled slider.
fn trait_slider(ui: &mut egui::Ui, npc: &mut NpcPersonality, kind: TraitKind) {
    let current = npc.get_trait(kind);
    let mut index = level_to_index(current) as f32;

    ui.horizontal(|ui| {
        ui.label(trait_label(kind));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(level_color(current), level_label(current));
            ui.add(egui::Slider::new(&mut index, 0.0..=4.0).show_value(false));
        });
    });

    let new_level = index_to_level(index as usize);
    if new_level != current {
        npc.set_trait(kind, new_level);
        tracing::debug!(?kind, ?new_level, "trait changed");
    }
}

/// Render the mood vector display.
fn mood_section(ui: &mut egui::Ui, mood: &bhava::mood::MoodVector) {
    egui::CollapsingHeader::new("Mood")
        .default_open(true)
        .show(ui, |ui| {
            mood_bar(ui, "Joy", mood.joy, egui::Color32::from_rgb(240, 200, 60));
            mood_bar(
                ui,
                "Arousal",
                mood.arousal,
                egui::Color32::from_rgb(220, 100, 60),
            );
            mood_bar(
                ui,
                "Dominance",
                mood.dominance,
                egui::Color32::from_rgb(100, 80, 200),
            );
            mood_bar(
                ui,
                "Trust",
                mood.trust,
                egui::Color32::from_rgb(60, 180, 120),
            );
            mood_bar(
                ui,
                "Interest",
                mood.interest,
                egui::Color32::from_rgb(80, 160, 220),
            );
            mood_bar(
                ui,
                "Frustration",
                mood.frustration,
                egui::Color32::from_rgb(200, 60, 60),
            );
        });
}

/// Render a horizontal mood bar (-1.0 to 1.0).
fn mood_bar(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(format!("{label:>12}"));

        let width = 120.0;
        let height = 12.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(40, 40, 45));

        // Center line
        let center_x = rect.center().x;
        painter.line_segment(
            [
                egui::pos2(center_x, rect.top()),
                egui::pos2(center_x, rect.bottom()),
            ],
            egui::Stroke::new(0.5, egui::Color32::from_rgb(80, 80, 80)),
        );

        // Value bar from center
        let bar_width = value.abs() * (width / 2.0);
        let bar_rect = if value >= 0.0 {
            egui::Rect::from_min_size(
                egui::pos2(center_x, rect.top()),
                egui::vec2(bar_width, height),
            )
        } else {
            egui::Rect::from_min_size(
                egui::pos2(center_x - bar_width, rect.top()),
                egui::vec2(bar_width, height),
            )
        };
        painter.rect_filled(bar_rect, 1.0, color);

        // Value text
        ui.label(format!("{value:+.2}"));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personality::ALL_TRAITS;

    #[test]
    fn level_to_index_roundtrip() {
        for (i, &level) in ALL_LEVELS.iter().enumerate() {
            assert_eq!(level_to_index(level), i);
            assert_eq!(index_to_level(i), level);
        }
    }

    #[test]
    fn all_traits_have_labels() {
        for kind in ALL_TRAITS {
            let label = trait_label(kind);
            assert!(!label.is_empty());
        }
    }

    #[test]
    fn all_levels_have_labels() {
        for level in ALL_LEVELS {
            let label = level_label(level);
            assert!(!label.is_empty());
        }
    }

    #[test]
    fn groups_cover_all_traits() {
        let mut covered = std::collections::HashSet::new();
        for group in GROUPS {
            for &kind in group.traits {
                covered.insert(kind);
            }
        }
        assert_eq!(covered.len(), 15);
    }

    #[test]
    fn index_to_level_clamps() {
        assert_eq!(index_to_level(999), TraitLevel::Highest);
    }

    #[test]
    fn level_colors_distinct() {
        let colors: Vec<_> = ALL_LEVELS.iter().map(|&l| level_color(l)).collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j]);
            }
        }
    }
}
