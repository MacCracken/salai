//! Texture utilities for the editor — thumbnail generation, color analysis, and
//! image inspection powered by [`ranga`].

use ranga::histogram;
use ranga::pixel::{PixelBuffer, PixelFormat};
use ranga::transform::{self, ScaleFilter};

/// Texture metadata gathered for the asset browser and inspector.
#[derive(Debug, Clone)]
pub struct TextureInfo {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format.
    pub format: String,
    /// Average color (sRGB).
    pub average_color: [u8; 4],
    /// File size in bytes (if known).
    pub file_size: Option<u64>,
}

/// Generate a thumbnail from raw RGBA pixel data.
///
/// Returns a new `PixelBuffer` resized to fit within `max_size x max_size`
/// while preserving aspect ratio.
#[must_use]
pub fn generate_thumbnail(
    data: Vec<u8>,
    width: u32,
    height: u32,
    max_size: u32,
) -> Option<PixelBuffer> {
    let buf = PixelBuffer::new(data, width, height, PixelFormat::Rgba8).ok()?;

    if width <= max_size && height <= max_size {
        return Some(buf);
    }

    let aspect = width as f32 / height as f32;
    let (tw, th) = if aspect >= 1.0 {
        (max_size, (max_size as f32 / aspect) as u32)
    } else {
        ((max_size as f32 * aspect) as u32, max_size)
    };

    transform::resize(&buf, tw.max(1), th.max(1), ScaleFilter::Bilinear).ok()
}

/// Compute the average color of an RGBA pixel buffer.
#[must_use]
pub fn average_color(data: &[u8], width: u32, height: u32) -> [u8; 4] {
    let pixel_count = (width as u64) * (height as u64);
    if pixel_count == 0 || data.len() < (pixel_count as usize * 4) {
        return [0, 0, 0, 255];
    }

    let (mut r, mut g, mut b, mut a) = (0u64, 0u64, 0u64, 0u64);
    for pixel in data.chunks_exact(4) {
        r += pixel[0] as u64;
        g += pixel[1] as u64;
        b += pixel[2] as u64;
        a += pixel[3] as u64;
    }

    [
        (r / pixel_count) as u8,
        (g / pixel_count) as u8,
        (b / pixel_count) as u8,
        (a / pixel_count) as u8,
    ]
}

/// Build a `TextureInfo` from raw RGBA pixel data.
#[must_use]
pub fn inspect_texture(data: &[u8], width: u32, height: u32) -> TextureInfo {
    TextureInfo {
        width,
        height,
        format: "RGBA8".into(),
        average_color: average_color(data, width, height),
        file_size: None,
    }
}

/// Compute a luminance histogram from raw RGBA pixel data.
///
/// Returns a normalized histogram with the specified number of bins,
/// or `None` if the buffer is invalid.
#[must_use]
pub fn luminance_histogram(
    data: Vec<u8>,
    width: u32,
    height: u32,
    bins: usize,
) -> Option<Vec<f64>> {
    let buf = PixelBuffer::new(data, width, height, PixelFormat::Rgba8).ok()?;
    histogram::luminance_histogram(&buf, bins).ok()
}

/// Convert an sRGB color to a display string.
#[must_use]
#[inline]
pub fn color_to_hex(color: [u8; 4]) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        color[0], color[1], color[2], color[3]
    )
}

/// Convert an sRGB [u8; 4] color to linear RGB [f32; 3].
#[must_use]
#[inline]
pub fn srgb_to_linear(color: [u8; 4]) -> [f32; 3] {
    [
        hisab::transforms::srgb_to_linear(color[0] as f32 / 255.0),
        hisab::transforms::srgb_to_linear(color[1] as f32 / 255.0),
        hisab::transforms::srgb_to_linear(color[2] as f32 / 255.0),
    ]
}

/// Convert a linear RGB [f32; 3] color to sRGB [u8; 4] with full alpha.
#[must_use]
#[inline]
pub fn linear_to_srgb(linear: [f32; 3]) -> [u8; 4] {
    [
        (hisab::transforms::linear_to_srgb(linear[0]) * 255.0).round() as u8,
        (hisab::transforms::linear_to_srgb(linear[1]) * 255.0).round() as u8,
        (hisab::transforms::linear_to_srgb(linear[2]) * 255.0).round() as u8,
        255,
    ]
}

/// Convert an sRGB [u8; 4] color to Oklab perceptual color space.
///
/// Returns `(L, a, b)` where L is lightness (0-1) and a,b are chroma axes.
#[must_use]
#[inline]
pub fn srgb_to_oklab(color: [u8; 4]) -> (f32, f32, f32) {
    let lin = srgb_to_linear(color);
    hisab::transforms::linear_to_oklab(lin[0], lin[1], lin[2])
}

/// Convert an Oklab `(L, a, b)` color to sRGB [u8; 4].
#[must_use]
#[inline]
pub fn oklab_to_srgb(l: f32, a: f32, b: f32) -> [u8; 4] {
    let (r, g, b) = hisab::transforms::oklab_to_linear(l, a, b);
    linear_to_srgb([r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white_2x2() -> Vec<u8> {
        vec![255u8; 2 * 2 * 4]
    }

    fn red_4x4() -> Vec<u8> {
        let mut data = Vec::with_capacity(4 * 4 * 4);
        for _ in 0..16 {
            data.extend_from_slice(&[255, 0, 0, 255]);
        }
        data
    }

    #[test]
    fn thumbnail_small_passthrough() {
        let data = white_2x2();
        let thumb = generate_thumbnail(data, 2, 2, 256).unwrap();
        assert_eq!(thumb.width, 2);
        assert_eq!(thumb.height, 2);
    }

    #[test]
    fn thumbnail_resize_preserves_aspect() {
        let data = vec![128u8; 100 * 50 * 4];
        let thumb = generate_thumbnail(data, 100, 50, 20).unwrap();
        assert!(thumb.width <= 20);
        assert!(thumb.height <= 20);
        // 2:1 aspect ratio
        assert!(thumb.width >= thumb.height);
    }

    #[test]
    fn average_color_solid_red() {
        let data = red_4x4();
        let avg = average_color(&data, 4, 4);
        assert_eq!(avg, [255, 0, 0, 255]);
    }

    #[test]
    fn average_color_solid_white() {
        let data = white_2x2();
        let avg = average_color(&data, 2, 2);
        assert_eq!(avg, [255, 255, 255, 255]);
    }

    #[test]
    fn average_color_empty() {
        let avg = average_color(&[], 0, 0);
        assert_eq!(avg, [0, 0, 0, 255]);
    }

    #[test]
    fn inspect_texture_basic() {
        let data = red_4x4();
        let info = inspect_texture(&data, 4, 4);
        assert_eq!(info.width, 4);
        assert_eq!(info.height, 4);
        assert_eq!(info.format, "RGBA8");
        assert_eq!(info.average_color, [255, 0, 0, 255]);
        assert!(info.file_size.is_none());
    }

    #[test]
    fn luminance_histogram_basic() {
        let data = white_2x2();
        let hist = luminance_histogram(data, 2, 2, 256).unwrap();
        assert_eq!(hist.len(), 256);
        // All white → all weight in the last bin
        assert!(hist[255] > 0.0);
    }

    #[test]
    fn color_to_hex_red() {
        assert_eq!(color_to_hex([255, 0, 0, 255]), "#FF0000FF");
    }

    #[test]
    fn color_to_hex_transparent() {
        assert_eq!(color_to_hex([0, 0, 0, 0]), "#00000000");
    }

    #[test]
    fn thumbnail_invalid_data() {
        // Data too short for claimed dimensions
        let result = generate_thumbnail(vec![0; 4], 100, 100, 50);
        assert!(result.is_none());
    }

    #[test]
    fn srgb_linear_roundtrip() {
        let original = [180, 60, 220, 255];
        let linear = srgb_to_linear(original);
        let back = linear_to_srgb(linear);
        // Allow ±1 for rounding
        for i in 0..3 {
            assert!((original[i] as i16 - back[i] as i16).abs() <= 1);
        }
    }

    #[test]
    fn oklab_roundtrip() {
        let original = [200, 100, 50, 255];
        let (l, a, b) = srgb_to_oklab(original);
        let back = oklab_to_srgb(l, a, b);
        for i in 0..3 {
            assert!((original[i] as i16 - back[i] as i16).abs() <= 2);
        }
    }

    #[test]
    fn srgb_to_linear_black_white() {
        let black = srgb_to_linear([0, 0, 0, 255]);
        assert!(black.iter().all(|&v| v < 0.001));

        let white = srgb_to_linear([255, 255, 255, 255]);
        assert!(white.iter().all(|&v| (v - 1.0).abs() < 0.01));
    }
}
