//! Animation timeline editor — keyframe editing, playback control, track display.
//!
//! Consumes soorat's `AnimationClip`, `AnimationChannel`, and `Keyframe` types.

use serde::{Deserialize, Serialize};

/// Animation editor state.
pub struct AnimationEditor {
    /// Currently loaded clip (editor's own representation).
    pub clip: Option<AnimClipState>,
    /// Playhead position in seconds.
    pub playhead: f32,
    /// Whether the animation is playing.
    pub playing: bool,
    /// Playback speed multiplier.
    pub speed: f32,
    /// Zoom level for the timeline (pixels per second).
    pub zoom: f32,
    /// Scroll offset in the timeline.
    pub scroll: f32,
}

impl Default for AnimationEditor {
    fn default() -> Self {
        Self {
            clip: None,
            playhead: 0.0,
            playing: false,
            speed: 1.0,
            zoom: 100.0,
            scroll: 0.0,
        }
    }
}

/// Editor representation of an animation clip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimClipState {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<AnimTrack>,
}

/// A single animation track (one property on one target).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimTrack {
    pub target: String,
    pub property: String,
    pub keyframes: Vec<AnimKeyframe>,
}

/// A keyframe in the editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimKeyframe {
    pub time: f32,
    pub values: Vec<f32>,
}

impl AnimationEditor {
    /// Load a soorat AnimationClip into the editor.
    pub fn load_clip(&mut self, clip: &soorat::animation::AnimationClip) {
        let tracks = clip
            .channels
            .iter()
            .map(|ch| AnimTrack {
                target: format!("Joint {}", ch.joint_index),
                property: format!("{:?}", ch.property),
                keyframes: ch
                    .keyframes
                    .iter()
                    .map(|kf| AnimKeyframe {
                        time: kf.time,
                        values: kf.value.clone(),
                    })
                    .collect(),
            })
            .collect();

        self.clip = Some(AnimClipState {
            name: clip.name.clone(),
            duration: clip.duration,
            tracks,
        });
        self.playhead = 0.0;
        self.playing = false;
        tracing::info!(name = %clip.name, duration = clip.duration, "animation clip loaded");
    }

    /// Advance the playhead by dt seconds (when playing).
    pub fn tick(&mut self, dt: f32) {
        if !self.playing {
            return;
        }
        if let Some(clip) = &self.clip
            && clip.duration > 0.0
        {
            self.playhead += dt * self.speed;
            if self.playhead > clip.duration {
                self.playhead %= clip.duration;
            }
        }
    }

    /// Add a keyframe to a track at the current playhead position.
    pub fn add_keyframe(&mut self, track_index: usize, values: Vec<f32>) {
        if let Some(clip) = &mut self.clip
            && let Some(track) = clip.tracks.get_mut(track_index)
        {
            track.keyframes.push(AnimKeyframe {
                time: self.playhead,
                values,
            });
            track.keyframes.sort_by(|a, b| a.time.total_cmp(&b.time));
            tracing::debug!(track = track_index, time = self.playhead, "keyframe added");
        }
    }

    /// Remove a keyframe from a track by index.
    pub fn remove_keyframe(&mut self, track_index: usize, keyframe_index: usize) {
        if let Some(clip) = &mut self.clip
            && let Some(track) = clip.tracks.get_mut(track_index)
            && keyframe_index < track.keyframes.len()
        {
            track.keyframes.remove(keyframe_index);
            tracing::debug!(
                track = track_index,
                keyframe = keyframe_index,
                "keyframe removed"
            );
        }
    }
}

/// Render the animation timeline panel.
pub fn animation_panel(ui: &mut egui::Ui, editor: &mut AnimationEditor) {
    ui.heading("Animation");
    ui.separator();

    // Transport controls
    ui.horizontal(|ui| {
        if ui
            .button(if editor.playing { "Pause" } else { "Play" })
            .clicked()
        {
            editor.playing = !editor.playing;
        }
        if ui.button("Stop").clicked() {
            editor.playing = false;
            editor.playhead = 0.0;
        }
        ui.separator();
        ui.label(format!("{:.2}s", editor.playhead));
        if let Some(clip) = &editor.clip {
            ui.label(format!("/ {:.2}s", clip.duration));
        }
        ui.separator();
        ui.label("Speed:");
        ui.add(
            egui::DragValue::new(&mut editor.speed)
                .speed(0.1)
                .range(0.1..=4.0),
        );
    });

    let Some(clip) = &editor.clip else {
        ui.label("No animation loaded");
        return;
    };

    // Playhead scrubber
    let mut playhead = editor.playhead;
    ui.add(egui::Slider::new(&mut playhead, 0.0..=clip.duration).text("Time"));
    editor.playhead = playhead;

    ui.separator();

    // Track list with keyframe indicators
    let duration = clip.duration;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for track in &clip.tracks {
            ui.horizontal(|ui| {
                ui.label(format!("{} — {}", track.target, track.property));
                ui.label(format!("({} keys)", track.keyframes.len()));
            });

            // Timeline bar for this track
            let width = ui.available_width();
            let height = 20.0;
            let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(35, 35, 40));

            // Keyframe diamonds
            for kf in &track.keyframes {
                if duration > 0.0 {
                    let x = rect.left() + (kf.time / duration) * rect.width();
                    let center = egui::pos2(x, rect.center().y);
                    let size = 4.0;
                    painter.add(egui::Shape::convex_polygon(
                        vec![
                            egui::pos2(center.x, center.y - size),
                            egui::pos2(center.x + size, center.y),
                            egui::pos2(center.x, center.y + size),
                            egui::pos2(center.x - size, center.y),
                        ],
                        egui::Color32::from_rgb(220, 180, 60),
                        egui::Stroke::NONE,
                    ));
                }
            }

            // Playhead line
            if duration > 0.0 {
                let px = rect.left() + (editor.playhead / duration) * rect.width();
                painter.line_segment(
                    [egui::pos2(px, rect.top()), egui::pos2(px, rect.bottom())],
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 200, 255)),
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_default() {
        let e = AnimationEditor::default();
        assert!(e.clip.is_none());
        assert!(!e.playing);
        assert_eq!(e.playhead, 0.0);
        assert_eq!(e.speed, 1.0);
    }

    #[test]
    fn tick_when_not_playing() {
        let mut e = AnimationEditor {
            clip: Some(AnimClipState {
                name: "test".into(),
                duration: 2.0,
                tracks: vec![],
            }),
            ..Default::default()
        };
        e.tick(0.1);
        assert_eq!(e.playhead, 0.0);
    }

    #[test]
    fn tick_advances_playhead() {
        let mut e = AnimationEditor {
            clip: Some(AnimClipState {
                name: "test".into(),
                duration: 2.0,
                tracks: vec![],
            }),
            playing: true,
            ..Default::default()
        };
        e.tick(0.5);
        assert!((e.playhead - 0.5).abs() < 0.01);
    }

    #[test]
    fn tick_loops() {
        let mut e = AnimationEditor {
            clip: Some(AnimClipState {
                name: "test".into(),
                duration: 1.0,
                tracks: vec![],
            }),
            playing: true,
            ..Default::default()
        };
        e.tick(1.5);
        assert!(e.playhead < 1.0);
    }

    #[test]
    fn add_keyframe() {
        let mut e = AnimationEditor {
            clip: Some(AnimClipState {
                name: "test".into(),
                duration: 2.0,
                tracks: vec![AnimTrack {
                    target: "Joint 0".into(),
                    property: "Translation".into(),
                    keyframes: vec![],
                }],
            }),
            playhead: 0.5,
            ..Default::default()
        };
        e.add_keyframe(0, vec![1.0, 2.0, 3.0]);
        assert_eq!(e.clip.as_ref().unwrap().tracks[0].keyframes.len(), 1);
        assert_eq!(e.clip.as_ref().unwrap().tracks[0].keyframes[0].time, 0.5);
    }

    #[test]
    fn remove_keyframe() {
        let mut e = AnimationEditor {
            clip: Some(AnimClipState {
                name: "test".into(),
                duration: 2.0,
                tracks: vec![AnimTrack {
                    target: "Joint 0".into(),
                    property: "Translation".into(),
                    keyframes: vec![
                        AnimKeyframe {
                            time: 0.0,
                            values: vec![0.0, 0.0, 0.0],
                        },
                        AnimKeyframe {
                            time: 1.0,
                            values: vec![1.0, 1.0, 1.0],
                        },
                    ],
                }],
            }),
            ..Default::default()
        };
        e.remove_keyframe(0, 0);
        assert_eq!(e.clip.as_ref().unwrap().tracks[0].keyframes.len(), 1);
        assert_eq!(e.clip.as_ref().unwrap().tracks[0].keyframes[0].time, 1.0);
    }

    #[test]
    fn add_keyframe_maintains_sort() {
        let mut e = AnimationEditor {
            clip: Some(AnimClipState {
                name: "test".into(),
                duration: 3.0,
                tracks: vec![AnimTrack {
                    target: "J0".into(),
                    property: "T".into(),
                    keyframes: vec![
                        AnimKeyframe {
                            time: 0.0,
                            values: vec![0.0],
                        },
                        AnimKeyframe {
                            time: 2.0,
                            values: vec![2.0],
                        },
                    ],
                }],
            }),
            playhead: 1.0,
            ..Default::default()
        };
        e.add_keyframe(0, vec![1.0]);
        let times: Vec<f32> = e.clip.as_ref().unwrap().tracks[0]
            .keyframes
            .iter()
            .map(|k| k.time)
            .collect();
        assert_eq!(times, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn speed_multiplier() {
        let mut e = AnimationEditor {
            clip: Some(AnimClipState {
                name: "test".into(),
                duration: 10.0,
                tracks: vec![],
            }),
            playing: true,
            speed: 2.0,
            ..Default::default()
        };
        e.tick(0.5);
        assert!((e.playhead - 1.0).abs() < 0.01);
    }

    #[test]
    fn tick_zero_duration_no_panic() {
        let mut e = AnimationEditor {
            clip: Some(AnimClipState {
                name: "zero".into(),
                duration: 0.0,
                tracks: vec![],
            }),
            playing: true,
            ..Default::default()
        };
        e.tick(1.0);
        assert_eq!(e.playhead, 0.0);
    }
}
