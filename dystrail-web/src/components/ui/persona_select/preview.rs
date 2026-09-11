use super::helpers::{localized_desc, localized_name, modifier_text, multiplier_label, stats_row};
use crate::game::personas::Persona;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct PersonaPreviewProps {
    pub persona: Option<Persona>,
}

#[function_component(PersonaPreview)]
pub fn persona_preview(props: &PersonaPreviewProps) -> Html {
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
                <p class="persona-origin">{crate::i18n::tr("route.origin",Some(&std::collections::BTreeMap::from([("city",crate::game::route::origin(&per.id))])))}</p>
                <p class="mission-copy">{crate::i18n::t("journey.mission")}</p>
                <div class="persona-mods">{ mods_text }</div>

                <div class="persona-preview-stats">
                  { stats_row(per) }
                </div>

                <p class="muted">{ mult_line }{" · "}{crate::i18n::t("journey.multiplier")}</p>

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
