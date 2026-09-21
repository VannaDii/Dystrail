//! Simulation-clock light profiles; menus and animations never advance scene time.
#[must_use]
pub const fn profile(hour: u8) -> &'static str {
    match hour % 24 {
        0..=5 | 21..=23 => "night",
        6..=9 => "morning",
        10..=14 => "day",
        15..=17 => "afternoon",
        _ => "dusk",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn end_of_driving_does_not_fake_sunset() {
        assert_eq!(profile(8), "morning");
        assert_eq!(profile(13), "day");
        assert_eq!(profile(16), "afternoon");
        assert_eq!(profile(19), "dusk");
        assert_eq!(profile(23), "night");
        assert_eq!(profile(24), "night");
    }
}
