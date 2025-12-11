use crate::game::{DietId, PaceId, PacingConfig};
use crate::i18n;

pub(super) const fn pace_code(pace: PaceId) -> &'static str {
    match pace {
        PaceId::Steady => "S",
        PaceId::Heated => "H",
        PaceId::Blitz => "B",
    }
}

pub(super) const fn diet_code(diet: DietId) -> &'static str {
    match diet {
        DietId::Mixed => "M",
        DietId::Quiet => "Q",
        DietId::Doom => "D",
    }
}

pub(super) fn pace_preview(pacing_config: &PacingConfig, pace: PaceId) -> String {
    pacing_config
        .pace
        .iter()
        .find(|p| p.id == pace.as_str())
        .map_or_else(String::new, |p| {
            format!(
                "{} | {}: {} {}: {}",
                p.name,
                i18n::t("stats.pants"),
                p.pants,
                i18n::t("stats.sanity_short"),
                p.sanity
            )
        })
}

pub(super) fn diet_preview(pacing_config: &PacingConfig, diet: DietId) -> String {
    pacing_config
        .diet
        .iter()
        .find(|d| d.id == diet.as_str())
        .map_or_else(String::new, |d| {
            format!(
                "{} | {}: {} {}: {}",
                d.name,
                i18n::t("stats.pants"),
                d.pants,
                i18n::t("stats.sanity_short"),
                d.sanity
            )
        })
}
