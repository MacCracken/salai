//! Audio asset preview and play-mode utilities powered by [`dhvani`].
//!
//! Provides audio inspection (waveform, loudness, duration) for the asset
//! browser and audio clock integration for play-mode simulation.

use dhvani::AudioBuffer;

/// Audio asset metadata for the inspector and asset browser.
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Number of audio channels.
    pub channels: u32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Peak amplitude (0.0 to 1.0).
    pub peak: f32,
    /// RMS level.
    pub rms: f32,
    /// Loudness in LUFS (if computed).
    pub loudness_lufs: Option<f32>,
}

/// Inspect an audio buffer and return metadata.
#[must_use]
pub fn inspect_audio(buf: &AudioBuffer) -> AudioInfo {
    AudioInfo {
        duration_secs: buf.duration_secs(),
        channels: buf.channels(),
        sample_rate: buf.sample_rate(),
        peak: buf.peak(),
        rms: buf.rms(),
        loudness_lufs: None,
    }
}

/// Inspect an audio buffer with loudness measurement (slower, EBU R128).
#[must_use]
pub fn inspect_audio_with_loudness(buf: &AudioBuffer) -> AudioInfo {
    let lufs = dhvani::analysis::loudness_lufs(buf);
    AudioInfo {
        duration_secs: buf.duration_secs(),
        channels: buf.channels(),
        sample_rate: buf.sample_rate(),
        peak: buf.peak(),
        rms: buf.rms(),
        loudness_lufs: Some(lufs),
    }
}

/// Generate waveform visualization data for an audio buffer.
///
/// Returns (min, max) pairs per channel at the specified resolution.
#[must_use]
pub fn waveform(buf: &AudioBuffer, peaks_per_second: u32) -> dhvani::analysis::WaveformData {
    dhvani::analysis::compute_waveform(buf, peaks_per_second)
}

/// Format duration as MM:SS.mmm.
#[must_use]
pub fn format_duration(secs: f64) -> String {
    let mins = (secs / 60.0) as u32;
    let remaining = secs - (mins as f64 * 60.0);
    format!("{mins}:{remaining:06.3}")
}

/// Format a peak/RMS value as dB.
#[must_use]
#[inline]
pub fn amplitude_to_db_str(amp: f32) -> String {
    if amp <= 0.0 {
        return "-inf dB".into();
    }
    let db = dhvani::amplitude_to_db(amp);
    format!("{db:.1} dB")
}

/// Create an audio buffer from interleaved f32 samples.
pub fn buffer_from_samples(
    samples: Vec<f32>,
    channels: u32,
    sample_rate: u32,
) -> Result<AudioBuffer, dhvani::NadaError> {
    AudioBuffer::from_interleaved(samples, channels, sample_rate)
}

/// Normalize an audio buffer to a target peak.
pub fn normalize(buf: &mut AudioBuffer, target_peak: f32) {
    dhvani::dsp::normalize(buf, target_peak);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine_buffer(freq: f32, duration_secs: f32, sample_rate: u32) -> AudioBuffer {
        let num_samples = (duration_secs * sample_rate as f32) as usize;
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * freq * t).sin()
            })
            .collect();
        AudioBuffer::from_interleaved(samples, 1, sample_rate).unwrap()
    }

    fn silence_buffer(frames: usize) -> AudioBuffer {
        AudioBuffer::silence(1, frames, 44100)
    }

    #[test]
    fn inspect_sine() {
        let buf = sine_buffer(440.0, 1.0, 44100);
        let info = inspect_audio(&buf);
        assert!((info.duration_secs - 1.0).abs() < 0.01);
        assert_eq!(info.channels, 1);
        assert_eq!(info.sample_rate, 44100);
        assert!(info.peak > 0.9);
        assert!(info.rms > 0.0);
        assert!(info.loudness_lufs.is_none());
    }

    #[test]
    fn inspect_with_loudness() {
        let buf = sine_buffer(440.0, 0.5, 44100);
        let info = inspect_audio_with_loudness(&buf);
        assert!(info.loudness_lufs.is_some());
    }

    #[test]
    fn inspect_silence() {
        let buf = silence_buffer(44100);
        let info = inspect_audio(&buf);
        assert_eq!(info.peak, 0.0);
        assert_eq!(info.rms, 0.0);
    }

    #[test]
    fn waveform_basic() {
        let buf = sine_buffer(440.0, 1.0, 44100);
        let wf = waveform(&buf, 100);
        assert!(!wf.channels.is_empty());
        assert_eq!(wf.peaks_per_second, 100);
    }

    #[test]
    fn format_duration_basic() {
        assert_eq!(format_duration(0.0), "0:00.000");
        assert_eq!(format_duration(1.5), "0:01.500");
        assert_eq!(format_duration(65.0), "1:05.000");
        assert_eq!(format_duration(125.5), "2:05.500");
    }

    #[test]
    fn amplitude_to_db_str_zero() {
        assert_eq!(amplitude_to_db_str(0.0), "-inf dB");
    }

    #[test]
    fn amplitude_to_db_str_unity() {
        let s = amplitude_to_db_str(1.0);
        assert!(s.contains("0.0"));
    }

    #[test]
    fn amplitude_to_db_str_half() {
        let s = amplitude_to_db_str(0.5);
        assert!(s.contains("-6")); // ~-6 dB
    }

    #[test]
    fn buffer_from_samples_valid() {
        let samples = vec![0.0f32; 44100];
        let buf = buffer_from_samples(samples, 1, 44100).unwrap();
        assert_eq!(buf.channels(), 1);
        assert_eq!(buf.sample_rate(), 44100);
    }

    #[test]
    fn normalize_increases_level() {
        let mut buf = sine_buffer(440.0, 0.1, 44100);
        buf.apply_gain(0.1); // make quiet
        let peak_before = buf.peak();
        normalize(&mut buf, 1.0);
        assert!(buf.peak() > peak_before);
    }
}
