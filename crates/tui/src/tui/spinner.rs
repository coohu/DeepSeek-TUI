//! Shared animation frames for running-state UI chrome.
//!
//! Keep the braille spinner in one place so transcript tool cards, sidebars,
//! and any future running-job surfaces advance with the same cadence.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Braille spinner frames used for running tools and background jobs.
pub(crate) const BRAILLE_SPINNER_FRAMES: [&str; 10] = [
    "\u{280B}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283C}", "\u{2834}", "\u{2826}", "\u{2827}",
    "\u{2807}", "\u{280F}",
];

/// Match the live UI repaint cadence so running glyphs advance on every tick.
pub(crate) const BRAILLE_SPINNER_FRAME_MS: u64 = 80;

#[must_use]
pub(crate) fn braille_spinner_frame_for_elapsed_ms(
    elapsed_ms: u128,
    low_motion: bool,
) -> &'static str {
    if low_motion {
        return BRAILLE_SPINNER_FRAMES[0];
    }
    let idx = elapsed_ms
        .checked_div(u128::from(BRAILLE_SPINNER_FRAME_MS))
        .map_or(0, |frame| frame % BRAILLE_SPINNER_FRAMES.len() as u128);
    BRAILLE_SPINNER_FRAMES[usize::try_from(idx).unwrap_or_default()]
}

#[must_use]
pub(crate) fn braille_spinner_frame_for_duration_ms(
    duration_ms: u64,
    low_motion: bool,
) -> &'static str {
    braille_spinner_frame_for_elapsed_ms(u128::from(duration_ms), low_motion)
}

#[must_use]
pub(crate) fn braille_spinner_frame(started_at: Option<Instant>, low_motion: bool) -> &'static str {
    let elapsed_ms = started_at.map_or_else(
        || {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_millis())
        },
        |started| started.elapsed().as_millis(),
    );
    braille_spinner_frame_for_elapsed_ms(elapsed_ms, low_motion)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braille_spinner_advances_at_shared_cadence() {
        assert_eq!(braille_spinner_frame_for_elapsed_ms(0, false), "\u{280B}");
        assert_eq!(
            braille_spinner_frame_for_elapsed_ms(u128::from(BRAILLE_SPINNER_FRAME_MS) - 1, false),
            "\u{280B}"
        );
        assert_eq!(
            braille_spinner_frame_for_elapsed_ms(u128::from(BRAILLE_SPINNER_FRAME_MS), false),
            "\u{2819}"
        );
    }

    #[test]
    fn braille_spinner_respects_low_motion() {
        assert_eq!(
            braille_spinner_frame_for_elapsed_ms(u128::from(BRAILLE_SPINNER_FRAME_MS) * 3, true),
            "\u{280B}"
        );
    }
}
