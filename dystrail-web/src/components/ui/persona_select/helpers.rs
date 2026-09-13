use crate::game::personas::Persona;
use rand::{Rng, SeedableRng};
use std::collections::BTreeMap;
use yew::prelude::*;

pub(super) fn initial_selection(
    personas: &[Persona],
    initial_id: Option<&str>,
    entropy: u64,
) -> Option<usize> {
    if personas.is_empty() {
        return None;
    }
    personas
        .iter()
        .position(|per| Some(per.id.as_str()) == initial_id)
        .or_else(|| {
            Some(rand_chacha::ChaCha8Rng::seed_from_u64(entropy).gen_range(0..personas.len()))
        })
}

pub(super) fn selection_summary(per: &Persona) -> String {
    crate::i18n::tr(
        "persona.selection_summary",
        Some(&BTreeMap::from([
            ("name", localized_name(per).as_str()),
            (
                "budget",
                crate::i18n::fmt_currency(i64::from(per.start.budget) * 100).as_str(),
            ),
        ])),
    )
}

pub(super) fn mini_stat(label: String, value: i32) -> Html {
    html! {
        <span class="mini-stat" role="text">
            <span class="mini-stat-icon" aria-hidden="true">{ label }</span>
            <span class="mini-stat-value">{ value }</span>
        </span>
    }
}

pub(super) fn localized_name(per: &Persona) -> String {
    let key = format!("persona.{}.name", per.id);
    let localized = crate::i18n::t(&key);
    if localized == key {
        per.name.clone()
    } else {
        localized
    }
}

pub(super) fn localized_desc(per: &Persona) -> String {
    let key = format!("persona.{}.desc", per.id);
    let localized = crate::i18n::t(&key);
    if localized == key {
        per.desc.clone()
    } else {
        localized
    }
}

pub(super) fn multiplier_label(per: &Persona) -> String {
    let mult = format!("{:.1}", per.score_mult);
    let mut m = BTreeMap::new();
    m.insert("mult", mult.as_str());
    crate::i18n::tr("persona.mult", Some(&m))
}

pub(super) fn modifier_text(per: &Persona) -> String {
    let mut lines: Vec<String> = vec![];
    if per.mods.receipt_find_pct != 0 {
        let pct = per.mods.receipt_find_pct.to_string();
        let mut m = BTreeMap::new();
        m.insert("pct", pct.as_str());
        lines.push(crate::i18n::tr("persona.mods.receipts_pct", Some(&m)));
    }
    if per.mods.store_discount_pct != 0 {
        let pct = per.mods.store_discount_pct.to_string();
        let mut m = BTreeMap::new();
        m.insert("pct", pct.as_str());
        lines.push(crate::i18n::tr("persona.mods.store_discount_pct", Some(&m)));
    }
    if per.mods.bribe_discount_pct != 0 {
        let pct = per.mods.bribe_discount_pct.to_string();
        let mut m = BTreeMap::new();
        m.insert("pct", pct.as_str());
        lines.push(crate::i18n::tr("persona.mods.bribe_discount_pct", Some(&m)));
    }
    if per.mods.eo_heat_pct != 0 {
        let pct = per.mods.eo_heat_pct.to_string();
        let mut m = BTreeMap::new();
        m.insert("pct", pct.as_str());
        lines.push(crate::i18n::tr("persona.mods.eo_heat_pct", Some(&m)));
    }
    if per.mods.satire_sustain {
        lines.push(crate::i18n::t("persona.mods.satire_sustain"));
    }
    lines.join(" · ")
}

pub(super) fn stats_row(per: &Persona) -> Html {
    let mut starting_state = crate::game::GameState::default();
    starting_state.apply_persona(per);
    let stats = starting_state.stats;
    html! {
        <>
            { mini_stat(crate::i18n::t("ux.sanity"), stats.sanity) }
            { mini_stat(crate::i18n::t("play.credibility"), stats.credibility) }
            { mini_stat(crate::i18n::t("play.morale"), stats.morale) }
            { mini_stat(crate::i18n::t("play.allies"), stats.allies) }
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::personas::PersonasList;
    #[test]
    fn initial_character_varies_but_recovery_preserves_the_selection() {
        let list =
            PersonasList::from_json(include_str!("../../../../static/assets/data/personas.json"))
                .unwrap();
        let choices: std::collections::BTreeSet<_> = (0..60)
            .filter_map(|seed| initial_selection(&list.0, None, seed))
            .collect();
        assert_eq!(choices.len(), list.0.len());
        for per in &list.0 {
            let i = initial_selection(&list.0, Some(&per.id), 91).unwrap();
            assert_eq!(list.0[i].id, per.id);
        }
        assert_eq!(initial_selection(&[], None, 1), None);
    }
}
