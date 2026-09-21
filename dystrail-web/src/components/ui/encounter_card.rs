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
        let mut effects = super::choice_effects::describe(&c.effects, &p.stats);
        if c.effects.add_receipt.is_some() && p.receipt_bonus_chance > 0 {
            effects.push(crate::i18n::t("qualitative.bonus_evidence"));
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
    fn encounter_renders_qualitative_effects_including_accessible_tooltips() {
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
            html.contains("Improves Supplies") || html.contains("Improves Sup"),
            "{html}"
        );
        assert!(html.contains("Costs San"), "{html}");
        assert!(html.contains("Collect evidence"), "{html}");
        assert!(html.contains("May uncover extra evidence"), "{html}");
        for forbidden in ["+2", "−1", "-1", "+1", "25%"] {
            assert!(
                !html.contains(forbidden),
                "numeric preview leaked: {forbidden}: {html}"
            );
        }
    }
}
