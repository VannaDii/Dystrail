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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::personas::{PersonaMods, PersonaStart};
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    fn persona() -> Persona {
        crate::i18n::set_lang("en");
        Persona {
            id: "tester".into(),
            name: "Tester".into(),
            desc: "Desc".into(),
            score_mult: 1.5,
            start: PersonaStart {
                supplies: 3,
                credibility: 2,
                sanity: 4,
                morale: 1,
                allies: 0,
                budget: 500,
            },
            mods: PersonaMods {
                receipt_find_pct: 5,
                store_discount_pct: 10,
                bribe_discount_pct: 15,
                eo_heat_pct: 20,
                satire_sustain: true,
                pants_relief: 0,
                pants_relief_threshold: 0,
            },
        }
    }

    #[test]
    fn initials_fall_back_for_empty_names() {
        assert_eq!(initial_for(""), "?");
        assert_eq!(initial_for("Ada"), "A");
    }

    #[test]
    fn localized_helpers_use_defaults_when_missing_keys() {
        let per = persona();
        assert_eq!(localized_name(&per), per.name);
        assert_eq!(localized_desc(&per), per.desc);
        assert!(multiplier_value(&per).contains("×1.5"));
        assert!(!multiplier_label(&per).is_empty());
        assert!(!preview_line(&per).is_empty());
        assert!(!modifier_text(&per).is_empty());
    }

    #[function_component(StatsRowHarness)]
    fn stats_row_harness() -> Html {
        stats_row(&persona())
    }

    #[test]
    fn stats_row_renders_html() {
        let html = block_on(LocalServerRenderer::<StatsRowHarness>::new().render());
        assert!(html.contains("mini-stat"));
    }
}
