use crate::game::data::Encounter;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub encounter: Encounter,
    pub on_choice: Callback<usize>,
    #[prop_or_default]
    pub stats: crate::game::Stats,
    #[prop_or_default]
    pub cash: i64,
    #[prop_or_default]
    pub receipts: usize,
    #[prop_or_default]
    pub receipt_bonus_chance: u8,
}

fn format_effects(effects: &crate::game::data::Effects, stats: &crate::game::Stats) -> Vec<String> {
    let mut lines = Vec::new();
    if effects.cash_cents != 0 {
        lines.push(format!(
            "{} {}{}",
            crate::i18n::t("play.cash"),
            if effects.cash_cents > 0 { "+" } else { "−" },
            crate::i18n::fmt_currency(effects.cash_cents.abs())
        ));
    }
    let mut capped = false;
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
            let actual = (current + base).clamp(0, cap) - current;
            capped |= actual != base;
            lines.push(format!("{} {actual:+}", crate::i18n::t(key)));
        }
    }
    if effects.add_receipt.is_some() {
        lines.push(format!("{} +1", crate::i18n::t("ux.receipt")));
    }
    if effects.use_receipt {
        lines.push(format!("{} −1", crate::i18n::t("ux.receipt")));
    }
    if capped {
        lines.push(crate::i18n::t("journey.capped"));
    }
    if effects.rest || effects.travel_bonus_ratio > 0.0 {
        lines.push(crate::i18n::t("journey.day_effects"));
    }
    lines
}

#[function_component(EncounterCard)]
pub fn encounter_card(p: &Props) -> Html {
    crate::i18n::use_language();
    let keyboard = {
        let cb = p.on_choice.clone();
        let count = p.encounter.choices.len();
        Callback::from(move |e: KeyboardEvent| {
            use wasm_bindgen::JsCast;
            if e.target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|el| matches!(el.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT"))
            {
                return;
            }
            if let Ok(n) = e.key().parse::<usize>()
                && n > 0
                && n <= count
            {
                e.prevent_default();
                cb.emit(n - 1);
            }
        })
    };
    let buttons = p.encounter.choices.iter().enumerate().map(|(i, c)| {
        let cb = {
            let on_choice = p.on_choice.clone();
            Callback::from(move |_| on_choice.emit(i))
        };
        let mut effects = format_effects(&c.effects, &p.stats);
        if c.effects.add_receipt.is_some() && p.receipt_bonus_chance > 0 {
            let chance = p.receipt_bonus_chance.to_string();
            let args = std::collections::BTreeMap::from([("chance", chance.as_str())]);
            effects.push(crate::i18n::tr("play.receipt_bonus", Some(&args)));
        }
        let tooltip = effects.join(" · ");
        let desc_id = format!("enc-choice-{i}-desc");
        html! {
            <div class="encounter-choice">
                <button
                    disabled={!c.effects.affordable(&p.stats,p.cash,p.receipts)}
                    onclick={cb}
                    class="retro-btn-choice action-button"
                    aria-label={format!("{}) {}",i+1,crate::i18n::encounter_text(&p.encounter.id,&format!("choice_{i}"),&c.label))}
                    aria-describedby={desc_id.clone()}
                    title={tooltip.clone()}
                    aria-keyshortcuts={format!("{}", i + 1)}
                >
                    <span class="choice-number" aria-hidden="true">{format!("{})", i + 1)}</span>
                    <span class="action-title">{ crate::i18n::encounter_text(&p.encounter.id, &format!("choice_{i}"), &c.label) }</span>
                <small id={desc_id} class="choice-effects action-detail">
                    { if tooltip.is_empty() { crate::i18n::t("ux.no_change") } else { tooltip } }
                </small></button>
            </div>
        }
    });
    html! {
        <section onkeydown={keyboard} class="panel retro-encounter encounter-panel" role="dialog" aria-modal="false" aria-labelledby="enc-title">
            <header class="section-header"><p class="eyebrow">{crate::i18n::t("ux.known_effects")} {super::satire_context::encounter(&p.encounter.id)}</p>
                <h2 class="sr-only" id="enc-title">{ crate::i18n::encounter_text(&p.encounter.id, "name", &p.encounter.name) }</h2>
            </header>
            <div class="encounter-desc">
                <p>{ crate::i18n::encounter_text(&p.encounter.id, "desc", &p.encounter.desc) }</p>
            </div>
            <footer class="panel-footer">
                { for buttons }
            </footer>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{Choice, Effects};
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn encounter_renders_effect_tooltips() {
        crate::i18n::set_lang("en");
        let encounter = Encounter {
            id: "test".into(),
            name: "Test".into(),
            desc: "Desc".into(),
            weight: 5,
            regions: vec![],
            modes: vec![],
            choices: vec![Choice {
                label: "Take supplies".into(),
                effects: Effects {
                    supplies: 2,
                    sanity: -1,
                    add_receipt: Some("testimony".into()),
                    ..Effects::default()
                },
            }],
            hard_stop: false,
            major_repair: false,
            chainable: false,
        };
        let html = block_on(
            LocalServerRenderer::<EncounterCard>::with_props(Props {
                encounter,
                on_choice: Callback::noop(),
                stats: crate::game::Stats::default(),
                cash: 0,
                receipts: 0,
                receipt_bonus_chance: 25,
            })
            .render(),
        );

        assert!(
            html.contains("Sup +2") || html.contains("Supplies +2"),
            "effects preview should show supplies delta: {html}"
        );
        assert!(
            html.contains("San") || html.contains("Sanity -1"),
            "effects preview should show sanity delta: {html}"
        );
        assert!(
            html.contains("Receipts +1"),
            "evidence must be offered before choosing: {html}"
        );
    }
}
