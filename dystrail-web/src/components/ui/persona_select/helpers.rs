use crate::game::personas::Persona;
use std::collections::BTreeMap;
use yew::prelude::*;

pub(super) fn initial_for(name: &str) -> String {
    name.chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>())
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

pub(super) fn multiplier_value(per: &Persona) -> String {
    format!("×{:.1}", per.score_mult)
}

pub(super) fn multiplier_label(per: &Persona) -> String {
    let mult = format!("{:.1}", per.score_mult);
    let mut m = BTreeMap::new();
    m.insert("mult", mult.as_str());
    crate::i18n::tr("persona.mult", Some(&m))
}

pub(super) fn preview_line(per: &Persona) -> String {
    let sup = per.start.supplies.to_string();
    let cred = per.start.credibility.to_string();
    let san = per.start.sanity.to_string();
    let mor = per.start.morale.to_string();
    let allies = per.start.allies.to_string();
    let budget = per.start.budget.to_string();
    let mut m = BTreeMap::new();
    m.insert("sup", sup.as_str());
    m.insert("cred", cred.as_str());
    m.insert("san", san.as_str());
    m.insert("mor", mor.as_str());
    m.insert("allies", allies.as_str());
    m.insert("budget", budget.as_str());
    crate::i18n::tr("persona.preview", Some(&m))
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
            { mini_stat(crate::i18n::t("stats.sup_short"), per.start.supplies) }
            { mini_stat(crate::i18n::t("stats.sanity_short"), per.start.sanity) }
            { mini_stat(crate::i18n::t("stats.cred_short"), per.start.credibility) }
            { mini_stat(crate::i18n::t("stats.mor_short"), per.start.morale) }
            { mini_stat(crate::i18n::t("stats.allies_short"), per.start.allies) }
            { mini_stat(crate::i18n::t("persona.selected_budget_prefix"), per.start.budget) }
        </>
    }
}
