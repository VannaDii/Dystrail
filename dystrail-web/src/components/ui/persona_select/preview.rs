use super::helpers::{localized_desc, localized_name, modifier_text, multiplier_label, stats_row};
use crate::game::personas::Persona;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct PersonaPreviewProps {
    pub persona: Option<Persona>,
}

#[function_component(PersonaPreview)]
pub fn persona_preview(props: &PersonaPreviewProps) -> Html {
    crate::i18n::use_language();
    let content = props.persona.as_ref().map_or_else(
        || html! { <p class="muted">{ crate::i18n::t("persona.preview_wait") }</p> },
        |per| {
            let name = localized_name(per);
            let desc = localized_desc(per);

            let mult_line = multiplier_label(per);
            let mods_text = modifier_text(per);
            html! {
              <>
                <div class="persona-preview-header">
                  <div>
                    <h3 class="persona-name">{ name.clone() }</h3>
                    <p class="muted">{ desc }</p>
                  </div>
                </div>
                <dl class="persona-starting"><div><dt>{crate::i18n::t("persona.starting_point")}</dt><dd>{crate::game::route::origin(&per.id)}</dd></div><div><dt>{crate::i18n::t("persona.selected_budget_prefix")}</dt><dd>{crate::i18n::fmt_currency(i64::from(per.start.budget) * 100)}</dd></div></dl>

                <div class="persona-preview-stats">
                  { stats_row(per) }
                </div>
                <div class="persona-mods"><h4>{crate::i18n::t("persona.strengths")}</h4><p>{if mods_text.is_empty(){crate::i18n::t("persona.no_modifiers")}else{mods_text}}</p></div>
                <p class="persona-scoring">{ mult_line }{" · "}{crate::i18n::t("journey.multiplier")}</p>

              </>
            }
        },
    );

    html! {
        <aside id="persona-preview" class="persona-preview-card" aria-live="polite">
            { content }
        </aside>
    }
}
