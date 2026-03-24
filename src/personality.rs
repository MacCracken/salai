//! NPC personality and emotion editing for the inspector.
//!
//! Wraps [`bhava`] types to provide editor-friendly personality authoring:
//! trait profiles with 15 dimensions, mood vectors, and preset templates.

use bhava::mood::MoodVector;
use bhava::traits::{PersonalityProfile, TraitGroup, TraitKind, TraitLevel};

/// Editor wrapper for an NPC's personality data.
#[derive(Debug, Clone)]
pub struct NpcPersonality {
    /// The bhava personality profile (15 trait dimensions).
    pub profile: PersonalityProfile,
    /// Current emotional state as a mood vector.
    pub mood: MoodVector,
}

impl NpcPersonality {
    /// Create a new NPC personality with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            profile: PersonalityProfile::new(name),
            mood: MoodVector::neutral(),
        }
    }

    /// Set a personality trait.
    pub fn set_trait(&mut self, kind: TraitKind, level: TraitLevel) {
        self.profile.set_trait(kind, level);
    }

    /// Get a personality trait level.
    pub fn get_trait(&self, kind: TraitKind) -> TraitLevel {
        self.profile.get_trait(kind)
    }

    /// Set all traits in a group to the same level.
    pub fn set_group(&mut self, group: TraitGroup, level: TraitLevel) {
        self.profile.set_group(group, level);
    }

    /// Get the average trait value for a group (-1.0 to 1.0).
    pub fn group_average(&self, group: TraitGroup) -> f32 {
        self.profile.group_average(group)
    }

    /// Get traits that differ from Balanced.
    pub fn active_traits(&self) -> Vec<(TraitKind, TraitLevel)> {
        self.profile
            .active_traits()
            .into_iter()
            .map(|tv| (tv.trait_name, tv.level))
            .collect()
    }

    /// Compatibility score with another NPC (0.0 to 1.0).
    pub fn compatibility(&self, other: &NpcPersonality) -> f32 {
        self.profile.compatibility(&other.profile)
    }

    /// Blend two personalities together (0.0 = self, 1.0 = other).
    pub fn blend(&self, other: &NpcPersonality, t: f32) -> NpcPersonality {
        NpcPersonality {
            profile: self.profile.blend(&other.profile, t),
            mood: MoodVector::neutral(),
        }
    }

    /// Gather inspector-displayable info about the personality.
    #[must_use]
    pub fn inspector_summary(&self) -> PersonalitySummary {
        let active = self.active_traits();
        PersonalitySummary {
            name: self.profile.name.clone(),
            trait_count: active.len(),
            dominant_group: dominant_group(&self.profile),
            mood_label: mood_label(&self.mood),
        }
    }
}

/// Summary for display in the inspector panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalitySummary {
    /// NPC name.
    pub name: String,
    /// Number of non-balanced traits.
    pub trait_count: usize,
    /// The trait group with the strongest average.
    pub dominant_group: String,
    /// Human-readable mood label.
    pub mood_label: String,
}

/// Find which trait group has the highest absolute average.
fn dominant_group(profile: &PersonalityProfile) -> String {
    let groups = [
        (TraitGroup::Social, "Social"),
        (TraitGroup::Cognitive, "Cognitive"),
        (TraitGroup::Behavioral, "Behavioral"),
        (TraitGroup::Professional, "Professional"),
    ];

    groups
        .iter()
        .max_by(|a, b| {
            profile
                .group_average(a.0)
                .abs()
                .partial_cmp(&profile.group_average(b.0).abs())
                .unwrap()
        })
        .map(|&(_, name)| name.to_string())
        .unwrap_or_else(|| "None".into())
}

/// Map a mood vector to a human-readable label.
fn mood_label(mood: &MoodVector) -> String {
    let dominant = [
        (mood.joy, "Joyful"),
        (-mood.joy, "Sad"),
        (mood.arousal, "Excited"),
        (-mood.arousal, "Calm"),
        (mood.trust, "Trusting"),
        (-mood.trust, "Guarded"),
        (mood.interest, "Curious"),
        (mood.frustration, "Frustrated"),
    ];

    dominant
        .iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .filter(|&&(v, _)| v > 0.1)
        .map(|&(_, label)| label.to_string())
        .unwrap_or_else(|| "Neutral".into())
}

/// All 15 trait kinds, useful for building editor UI loops.
pub const ALL_TRAITS: [TraitKind; 15] = [
    TraitKind::Formality,
    TraitKind::Humor,
    TraitKind::Verbosity,
    TraitKind::Directness,
    TraitKind::Warmth,
    TraitKind::Empathy,
    TraitKind::Patience,
    TraitKind::Confidence,
    TraitKind::Creativity,
    TraitKind::RiskTolerance,
    TraitKind::Curiosity,
    TraitKind::Skepticism,
    TraitKind::Autonomy,
    TraitKind::Pedagogy,
    TraitKind::Precision,
];

/// All trait levels in order, useful for building dropdowns.
pub const ALL_LEVELS: [TraitLevel; 5] = [
    TraitLevel::Lowest,
    TraitLevel::Low,
    TraitLevel::Balanced,
    TraitLevel::High,
    TraitLevel::Highest,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_npc_personality() {
        let npc = NpcPersonality::new("Guard");
        assert_eq!(npc.profile.name, "Guard");
        // MoodVector doesn't impl PartialEq, check fields are zero
        assert_eq!(npc.mood.joy, 0.0);
        assert_eq!(npc.mood.arousal, 0.0);
        assert_eq!(npc.mood.frustration, 0.0);
    }

    #[test]
    fn set_and_get_trait() {
        let mut npc = NpcPersonality::new("Hero");
        npc.set_trait(TraitKind::Confidence, TraitLevel::Highest);
        assert_eq!(npc.get_trait(TraitKind::Confidence), TraitLevel::Highest);
    }

    #[test]
    fn default_traits_are_balanced() {
        let npc = NpcPersonality::new("NPC");
        for kind in ALL_TRAITS {
            assert_eq!(npc.get_trait(kind), TraitLevel::Balanced);
        }
    }

    #[test]
    fn active_traits_empty_for_default() {
        let npc = NpcPersonality::new("Default");
        assert!(npc.active_traits().is_empty());
    }

    #[test]
    fn active_traits_tracks_changes() {
        let mut npc = NpcPersonality::new("Test");
        npc.set_trait(TraitKind::Humor, TraitLevel::High);
        npc.set_trait(TraitKind::Warmth, TraitLevel::Low);
        let active = npc.active_traits();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn set_group() {
        let mut npc = NpcPersonality::new("Warrior");
        npc.set_group(TraitGroup::Cognitive, TraitLevel::High);
        assert_eq!(npc.get_trait(TraitKind::Confidence), TraitLevel::High);
        assert_eq!(npc.get_trait(TraitKind::Creativity), TraitLevel::High);
    }

    #[test]
    fn group_average() {
        let mut npc = NpcPersonality::new("Test");
        npc.set_group(TraitGroup::Social, TraitLevel::High);
        let avg = npc.group_average(TraitGroup::Social);
        assert!(avg > 0.0);
    }

    #[test]
    fn compatibility_with_self() {
        let npc = NpcPersonality::new("Self");
        let compat = npc.compatibility(&npc);
        assert!((compat - 1.0).abs() < 0.01);
    }

    #[test]
    fn compatibility_with_opposite() {
        let mut a = NpcPersonality::new("A");
        let mut b = NpcPersonality::new("B");
        for kind in ALL_TRAITS {
            a.set_trait(kind, TraitLevel::Highest);
            b.set_trait(kind, TraitLevel::Lowest);
        }
        let compat = a.compatibility(&b);
        assert!(compat < 0.5);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = NpcPersonality::new("A");
        let mut b = NpcPersonality::new("B");
        a.set_trait(TraitKind::Humor, TraitLevel::Lowest);
        b.set_trait(TraitKind::Humor, TraitLevel::Highest);
        let blended = a.blend(&b, 0.5);
        // Midpoint should be near Balanced
        assert_eq!(blended.get_trait(TraitKind::Humor), TraitLevel::Balanced);
    }

    #[test]
    fn inspector_summary_default() {
        let npc = NpcPersonality::new("Villager");
        let summary = npc.inspector_summary();
        assert_eq!(summary.name, "Villager");
        assert_eq!(summary.trait_count, 0);
        assert_eq!(summary.mood_label, "Neutral");
    }

    #[test]
    fn inspector_summary_with_traits() {
        let mut npc = NpcPersonality::new("Boss");
        npc.set_trait(TraitKind::Confidence, TraitLevel::Highest);
        npc.set_trait(TraitKind::Warmth, TraitLevel::Lowest);
        let summary = npc.inspector_summary();
        assert_eq!(summary.trait_count, 2);
    }

    #[test]
    fn mood_label_neutral() {
        let mood = MoodVector::neutral();
        assert_eq!(mood_label(&mood), "Neutral");
    }

    #[test]
    fn mood_label_joyful() {
        let mut mood = MoodVector::neutral();
        mood.joy = 0.8;
        assert_eq!(mood_label(&mood), "Joyful");
    }

    #[test]
    fn mood_label_frustrated() {
        let mut mood = MoodVector::neutral();
        mood.frustration = 0.9;
        assert_eq!(mood_label(&mood), "Frustrated");
    }

    #[test]
    fn dominant_group_detection() {
        let mut npc = NpcPersonality::new("Test");
        npc.set_group(TraitGroup::Cognitive, TraitLevel::Highest);
        let summary = npc.inspector_summary();
        assert_eq!(summary.dominant_group, "Cognitive");
    }

    #[test]
    fn all_traits_count() {
        assert_eq!(ALL_TRAITS.len(), 15);
    }

    #[test]
    fn all_levels_count() {
        assert_eq!(ALL_LEVELS.len(), 5);
    }

    #[test]
    fn mood_label_sad() {
        let mut mood = MoodVector::neutral();
        mood.joy = -0.8;
        assert_eq!(mood_label(&mood), "Sad");
    }

    #[test]
    fn mood_label_excited() {
        let mut mood = MoodVector::neutral();
        mood.arousal = 0.7;
        assert_eq!(mood_label(&mood), "Excited");
    }

    #[test]
    fn blend_extremes() {
        let mut a = NpcPersonality::new("A");
        let b = NpcPersonality::new("B");
        a.set_trait(TraitKind::Confidence, TraitLevel::Highest);

        // t=0.0 should keep A's traits
        let blend_a = a.blend(&b, 0.0);
        assert_eq!(
            blend_a.get_trait(TraitKind::Confidence),
            TraitLevel::Highest
        );

        // t=1.0 should take B's traits
        let blend_b = a.blend(&b, 1.0);
        assert_eq!(
            blend_b.get_trait(TraitKind::Confidence),
            TraitLevel::Balanced
        );
    }

    #[test]
    fn group_average_default_is_zero() {
        let npc = NpcPersonality::new("Default");
        assert_eq!(npc.group_average(TraitGroup::Social), 0.0);
        assert_eq!(npc.group_average(TraitGroup::Cognitive), 0.0);
    }
}
