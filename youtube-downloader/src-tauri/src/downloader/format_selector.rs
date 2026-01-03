// FormatSelector - unified format selection logic
//
// Converts raw formats from InfoExtractor into UI-friendly options.
// Handles:
// - Best quality detection (by resolution/bitrate)
// - Standard resolution mapping (1080p, 720p, 480p, 360p)
// - Audio-only extraction
// - Size estimation (video + audio combined)
// - Codec preferences (H.264 for compatibility)
//
// NOTE: This module is prepared for future integration.
// Currently the app uses extract_format_options() in ytdlp.rs directly.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use super::extractors::ExtendedFormat;

/// Quality option for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityOption {
    /// Display label (e.g., "1080p (1920x1080)")
    pub label: String,

    /// Value for selection (e.g., "1080p", "best", "audio")
    pub value: String,

    /// yt-dlp format specification
    pub format_spec: String,

    /// Estimated file size (e.g., "150 MB")
    pub estimated_size: Option<String>,

    /// Codec info (e.g., "H.264" or "VP9")
    pub codec_info: Option<String>,

    /// Resolution in pixels (e.g., 1080)
    pub height: Option<u32>,

    /// Bitrate in kbps (for audio)
    pub bitrate: Option<f32>,

    /// Whether this is audio-only
    pub is_audio: bool,

    /// Whether this is recommended (best compatible)
    pub is_recommended: bool,
}

/// Format selector with smart quality detection
pub struct FormatSelector;

impl FormatSelector {
    /// Build quality options from raw formats
    pub fn build_quality_options(formats: &[ExtendedFormat]) -> Vec<QualityOption> {
        let mut options = Vec::new();

        // Separate video and audio formats
        let video_formats: Vec<&ExtendedFormat> = formats
            .iter()
            .filter(|f| {
                f.vcodec
                    .as_ref()
                    .map_or(false, |v| v != "none" && !v.is_empty())
            })
            .collect();

        let audio_formats: Vec<&ExtendedFormat> = formats
            .iter()
            .filter(|f| {
                f.vcodec.as_ref().map_or(true, |v| v == "none")
                    && f.acodec
                        .as_ref()
                        .map_or(false, |a| a != "none" && !a.is_empty())
            })
            .collect();

        // Get best audio for size calculation
        let best_audio = Self::find_best_audio(&audio_formats);
        let best_audio_size = best_audio.and_then(|a| a.effective_size()).unwrap_or(0);

        // 1. Best Quality option
        if let Some(best) = Self::find_best_video(&video_formats) {
            let total_size = best
                .effective_size()
                .map(|s| s + best_audio_size)
                .or(Some(best_audio_size));

            let codec_label = Self::get_codec_label(best);
            let is_h264 = best.is_h264();

            options.push(QualityOption {
                label: format!(
                    "Best Quality ({}x{})",
                    best.width.unwrap_or(0),
                    best.height.unwrap_or(0)
                ),
                value: "best".to_string(),
                format_spec: "bv*+ba/best".to_string(),
                estimated_size: Self::format_size(total_size),
                codec_info: Some(codec_label),
                height: best.height,
                bitrate: best.tbr,
                is_audio: false,
                is_recommended: is_h264, // H.264 is most compatible
            });
        } else {
            // Fallback if no video found
            options.push(QualityOption {
                label: "Best Quality".to_string(),
                value: "best".to_string(),
                format_spec: "bv*+ba/best".to_string(),
                estimated_size: None,
                codec_info: None,
                height: None,
                bitrate: None,
                is_audio: false,
                is_recommended: true,
            });
        }

        // 2. Standard resolutions
        let resolutions = [
            ("1080p", 1080),
            ("720p", 720),
            ("480p", 480),
            ("360p", 360),
        ];

        for (label, target_height) in resolutions {
            if let Some(fmt) = Self::find_by_height(&video_formats, target_height) {
                // Skip if same as best quality
                if options
                    .first()
                    .map_or(false, |b| b.height == fmt.height)
                {
                    continue;
                }

                let total_size = fmt
                    .effective_size()
                    .map(|s| s + best_audio_size)
                    .or(Some(best_audio_size));

                let codec_label = Self::get_codec_label(fmt);

                options.push(QualityOption {
                    label: format!(
                        "{} ({}x{})",
                        label,
                        fmt.width.unwrap_or(0),
                        fmt.height.unwrap_or(0)
                    ),
                    value: label.to_string(),
                    format_spec: format!(
                        "bv*[height<={}]+ba/b[height<={}]/bv*+ba/best",
                        target_height, target_height
                    ),
                    estimated_size: Self::format_size(total_size),
                    codec_info: Some(codec_label),
                    height: fmt.height,
                    bitrate: fmt.tbr,
                    is_audio: false,
                    is_recommended: fmt.is_h264(),
                });
            }
        }

        // 3. Audio only (MP3)
        if let Some(audio) = best_audio {
            options.push(QualityOption {
                label: "Audio Only (MP3)".to_string(),
                value: "audio".to_string(),
                format_spec: "ba/b".to_string(),
                estimated_size: Self::format_size(audio.effective_size()),
                codec_info: audio.acodec.clone(),
                height: None,
                bitrate: audio.abr,
                is_audio: true,
                is_recommended: false,
            });
        } else {
            options.push(QualityOption {
                label: "Audio Only (MP3)".to_string(),
                value: "audio".to_string(),
                format_spec: "ba/b".to_string(),
                estimated_size: None,
                codec_info: None,
                height: None,
                bitrate: None,
                is_audio: true,
                is_recommended: false,
            });
        }

        options
    }

    /// Find best video format (highest resolution with H.264 preference)
    fn find_best_video<'a>(formats: &[&'a ExtendedFormat]) -> Option<&'a ExtendedFormat> {
        // First try to find best H.264 (most compatible)
        let best_h264 = formats
            .iter()
            .filter(|f| f.is_h264())
            .max_by_key(|f| f.height.unwrap_or(0));

        if let Some(h264) = best_h264 {
            // Check if there's a significantly higher resolution in other codecs
            let best_any = formats.iter().max_by_key(|f| f.height.unwrap_or(0));

            if let Some(any) = best_any {
                // If VP9/AV1 is much higher resolution (e.g., 4K vs 1080p), use that
                let h264_height = h264.height.unwrap_or(0);
                let any_height = any.height.unwrap_or(0);

                if any_height > h264_height * 3 / 2 {
                    return Some(any);
                }
            }

            return Some(h264);
        }

        // Fallback to any highest resolution
        formats.iter().max_by_key(|f| f.height.unwrap_or(0)).copied()
    }

    /// Find video format by target height (within 10% tolerance)
    fn find_by_height<'a>(
        formats: &[&'a ExtendedFormat],
        target_height: u32,
    ) -> Option<&'a ExtendedFormat> {
        let min_height = target_height * 9 / 10;
        let max_height = target_height * 11 / 10;

        let matches: Vec<&&ExtendedFormat> = formats
            .iter()
            .filter(|f| {
                f.height
                    .map_or(false, |h| h >= min_height && h <= max_height)
            })
            .collect();

        // Prefer H.264 among matches
        let h264_match = matches.iter().find(|f| f.is_h264());
        if let Some(h264) = h264_match {
            return Some(h264);
        }

        // Otherwise return highest bitrate match
        matches
            .iter()
            .max_by_key(|f| f.effective_size().unwrap_or(0))
            .copied()
            .copied()
    }

    /// Find best audio format (prefer AAC for compatibility)
    fn find_best_audio<'a>(formats: &[&'a ExtendedFormat]) -> Option<&'a ExtendedFormat> {
        // Prefer AAC (m4a) for compatibility
        let aac = formats
            .iter()
            .filter(|f| f.is_aac())
            .max_by_key(|f| f.abr.map(|b| (b * 100.0) as u32).unwrap_or(0));

        if let Some(a) = aac {
            return Some(a);
        }

        // Fallback to highest bitrate audio
        formats
            .iter()
            .max_by_key(|f| f.abr.map(|b| (b * 100.0) as u32).unwrap_or(0))
            .copied()
    }

    /// Format file size for display
    fn format_size(bytes: Option<u64>) -> Option<String> {
        bytes.map(|b| {
            let mb = b as f64 / 1_048_576.0;
            if mb >= 1024.0 {
                format!("{:.1} GB", mb / 1024.0)
            } else {
                format!("{:.0} MB", mb)
            }
        })
    }

    /// Get human-readable codec label
    fn get_codec_label(format: &ExtendedFormat) -> String {
        if format.is_h264() {
            "H.264".to_string()
        } else if format.is_vp9() {
            "VP9".to_string()
        } else if format.is_av1() {
            "AV1".to_string()
        } else {
            format
                .vcodec
                .as_ref()
                .map(|v| {
                    if v.contains('.') {
                        v.split('.').next().unwrap_or(v).to_string()
                    } else {
                        v.clone()
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string())
        }
    }

    /// Get format spec for yt-dlp based on quality value
    pub fn get_format_spec(quality: &str) -> String {
        match quality {
            "best" => "bv*+ba/best".to_string(),
            "1080p" => "bv*[height<=1080]+ba/b[height<=1080]/bv*+ba/best".to_string(),
            "720p" => "bv*[height<=720]+ba/b[height<=720]/bv*+ba/best".to_string(),
            "480p" => "bv*[height<=480]+ba/b[height<=480]/bv*+ba/best".to_string(),
            "360p" => "bv*[height<=360]+ba/b[height<=360]/bv*+ba/best".to_string(),
            "audio" => "ba/b".to_string(),
            _ => "bv*+ba/best".to_string(),
        }
    }

    /// Get recommended quality for a video
    pub fn recommend_quality(formats: &[ExtendedFormat]) -> &'static str {
        // Find max available height
        let max_height = formats
            .iter()
            .filter_map(|f| f.height)
            .max()
            .unwrap_or(0);

        // Recommend based on available quality
        if max_height >= 1080 {
            "1080p" // Good balance of quality and size
        } else if max_height >= 720 {
            "720p"
        } else if max_height >= 480 {
            "480p"
        } else {
            "best"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_video_format(height: u32, vcodec: &str, size: u64) -> ExtendedFormat {
        ExtendedFormat {
            format_id: format!("{}p", height),
            ext: "mp4".to_string(),
            resolution: Some(format!("{}x{}", height * 16 / 9, height)),
            width: Some(height * 16 / 9),
            height: Some(height),
            fps: Some(30.0),
            vcodec: Some(vcodec.to_string()),
            acodec: Some("none".to_string()),
            filesize: Some(size),
            filesize_approx: None,
            tbr: None,
            abr: None,
            vbr: None,
            format_note: None,
            video_only: true,
            audio_only: false,
        }
    }

    fn make_audio_format(bitrate: f32, size: u64) -> ExtendedFormat {
        ExtendedFormat {
            format_id: "audio".to_string(),
            ext: "m4a".to_string(),
            resolution: None,
            width: None,
            height: None,
            fps: None,
            vcodec: Some("none".to_string()),
            acodec: Some("mp4a.40.2".to_string()),
            filesize: Some(size),
            filesize_approx: None,
            tbr: None,
            abr: Some(bitrate),
            vbr: None,
            format_note: None,
            video_only: false,
            audio_only: true,
        }
    }

    #[test]
    fn test_quality_options_generation() {
        let formats = vec![
            make_video_format(1080, "avc1.4d401f", 100_000_000),
            make_video_format(720, "avc1.4d401e", 50_000_000),
            make_video_format(480, "avc1.4d401e", 25_000_000),
            make_audio_format(128.0, 5_000_000),
        ];

        let options = FormatSelector::build_quality_options(&formats);

        assert!(options.len() >= 3); // Best, at least one resolution, audio
        assert_eq!(options[0].value, "best");
        assert!(options.last().unwrap().is_audio);
    }

    #[test]
    fn test_h264_preference() {
        let formats = vec![
            make_video_format(1080, "vp9", 150_000_000),
            make_video_format(1080, "avc1.4d401f", 100_000_000),
        ];

        let refs: Vec<&ExtendedFormat> = formats.iter().collect();
        let best = FormatSelector::find_best_video(&refs);

        assert!(best.is_some());
        assert!(best.unwrap().is_h264());
    }
}

