use crate::game::personas::Persona;
use std::collections::BTreeMap;
use yew::prelude::*;

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
    html! {
        <>
            { mini_stat(crate::i18n::t("ux.sanity"), per.start.sanity) }
            { mini_stat(crate::i18n::t("play.credibility"), per.start.credibility) }
            { mini_stat(crate::i18n::t("play.morale"), per.start.morale) }
            { mini_stat(crate::i18n::t("play.allies"), per.start.allies) }
            { mini_stat(crate::i18n::t("persona.selected_budget_prefix"), per.start.budget) }
        </>
    }
}
