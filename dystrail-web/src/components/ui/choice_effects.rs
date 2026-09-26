//! Qualitative previews only; actual results remain numeric.
pub(crate) fn qualitative_stat(stat_key: &str, direction: &str) -> String {
    let stat = crate::i18n::t(stat_key);
    let args = std::collections::BTreeMap::from([("stat", stat.as_str())]);
    crate::i18n::tr(&format!("qualitative.{direction}"), Some(&args))
}

pub fn describe(effects: &crate::game::data::Effects, stats: &crate::game::Stats) -> Vec<String> {
    let mut lines = Vec::new();
    if effects.cash_cents != 0 {
        lines.push(qualitative_stat(
            "play.cash",
            if effects.cash_cents > 0 {
                "gain"
            } else {
                "cost"
            },
        ));
    }
    for (key, base, current, cap) in [
        ("ux.supplies", effects.supplies, stats.supplies, 20),
        ("ux.health", effects.hp, stats.hp, 10),
        ("ux.sanity", effects.sanity, stats.sanity, 10),
        (
            "play.credibility",
            effects.credibility,
            stats.credibility,
            20,
        ),
        ("play.morale", effects.morale, stats.morale, 10),
        ("play.allies", effects.allies, stats.allies, 50),
    ] {
        if base != 0 {
            let direction = if base > 0 {
                if current >= cap { "full" } else { "gain" }
            } else if current <= 0 {
                "empty"
            } else {
                "cost"
            };
            lines.push(qualitative_stat(key, direction));
        }
    }
    if effects.add_receipt.is_some() {
        lines.push(crate::i18n::t("qualitative.evidence"));
    }
    if effects.use_receipt {
        lines.push(crate::i18n::t("qualitative.use_evidence"));
    }
    if effects.rest || effects.travel_bonus_ratio > 0.0 {
        lines.push(crate::i18n::t("journey.day_effects"));
    }
    lines
}
