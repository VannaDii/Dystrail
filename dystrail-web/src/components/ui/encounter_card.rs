use crate::game::data::Encounter;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub encounter: Encounter,
    pub on_choice: Callback<usize>,
}

fn format_effects(effects: &crate::game::data::Effects) -> Vec<String> {
    let mut lines = Vec::new();
    if effects.supplies != 0 {
        lines.push(format!(
            "{} {:+}",
            crate::i18n::t("stats.sup_short"),
            effects.supplies
        ));
    }
    if effects.hp != 0 {
        lines.push(format!(
            "{} {:+}",
            crate::i18n::t("stats.hp_short"),
            effects.hp
        ));
    }
    if effects.sanity != 0 {
        lines.push(format!(
            "{} {:+}",
            crate::i18n::t("stats.sanity_short"),
            effects.sanity
        ));
    }
    if effects.credibility != 0 {
        lines.push(format!(
            "{} {:+}",
            crate::i18n::t("stats.cred_short"),
            effects.credibility
        ));
    }
    if effects.morale != 0 {
        lines.push(format!(
            "{} {:+}",
            crate::i18n::t("stats.mor_short"),
            effects.morale
        ));
    }
    if effects.allies != 0 {
        lines.push(format!(
            "{} {:+}",
            crate::i18n::t("stats.allies_short"),
            effects.allies
        ));
    }
    if effects.pants != 0 {
        lines.push(format!("Pants {:+}", effects.pants));
    }
    if effects.add_receipt.is_some() {
        lines.push("Receipts +1".to_string());
    }
    lines
}

#[function_component(EncounterCard)]
pub fn encounter_card(p: &Props) -> Html {
    let buttons = p.encounter.choices.iter().enumerate().map(|(i, c)| {
        let cb = {
            let on_choice = p.on_choice.clone();
            Callback::from(move |_| on_choice.emit(i))
        };
        let effects = format_effects(&c.effects);
        let tooltip = effects.join(" · ");
        let desc_id = format!("enc-choice-{i}-desc");
        html! {
            <div class="encounter-choice">
                <button
                    onclick={cb}
                    class="retro-btn-choice"
                    aria-describedby={desc_id.clone()}
                    title={tooltip.clone()}
                    aria-keyshortcuts={format!("{}", i + 1)}
                >
                    { format!("{}{}", i + 1, ") ") }{ c.label.clone() }
                </button>
                <div id={desc_id} class="sr-only">
                    { if tooltip.is_empty() { String::from("No change") } else { tooltip } }
                </div>
            </div>
        }
    });
    html! {
        <section class="panel retro-encounter encounter-panel" role="dialog" aria-modal="false" aria-labelledby="enc-title">
            <header class="section-header">
                <h2 id="enc-title">{ p.encounter.name.clone() }</h2>
            </header>
            <div class="encounter-desc">
                <p>{ p.encounter.desc.clone() }</p>
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
    }
}
