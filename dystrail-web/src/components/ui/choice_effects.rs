//! Qualitative previews only; actual results remain numeric.
pub(crate) fn qualitative_stat(stat_key: &str, direction: &str) -> String {
    let stat = crate::i18n::t(stat_key);
    qualitative_label(&stat, direction)
}

fn qualitative_label(stat: &str, direction: &str) -> String {
    let args = std::collections::BTreeMap::from([("stat", stat)]);
    crate::i18n::tr(&format!("qualitative.{direction}"), Some(&args))
}

pub fn describe(effects: &crate::game::data::Effects, stats: &crate::game::Stats) -> Vec<String> {
    let mut lines = Vec::new();
    if effects.cash_cents != 0 {
        let amount = crate::i18n::fmt_currency(effects.cash_cents.saturating_abs());
        lines.push(qualitative_label(
            &format!("{} ({amount})", crate::i18n::t("play.cash")),
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

#[cfg(test)]
mod tests {
    #[test]
    fn cash_previews_keep_the_actual_price_or_payment() {
        crate::i18n::set_lang("en");
        let stats = crate::game::Stats::default();
        for cents in [-700, 1800] {
            let effect = crate::game::data::Effects {
                cash_cents: cents,
                ..Default::default()
            };
            let preview = super::describe(&effect, &stats).join(" ");
            assert!(preview.contains(&crate::i18n::fmt_currency(cents.abs())));
        }
    }
}
