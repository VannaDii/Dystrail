use super::helpers::{
    initial_for, localized_desc, localized_name, modifier_text, multiplier_label, preview_line,
    stats_row,
};
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
            let preview_line = preview_line(per);
            let mult_line = multiplier_label(per);
            let mods_text = modifier_text(per);
            html! {
              <>
                <div class="persona-preview-header">
                  <div class="persona-portrait" aria-hidden="true">
                    <span class="portrait-initial">{ initial_for(&name) }</span>
                  </div>
                  <div>
                    <h3 class="persona-name">{ name.clone() }</h3>
                    <p class="muted">{ desc }</p>
                  </div>
                </div>
                <div class="persona-preview-stats">
                  { stats_row(per) }
                </div>
                <p class="muted">{ preview_line }</p>
                <p class="muted">{ mult_line }</p>
                <div class="persona-mods">{ mods_text }</div>
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
