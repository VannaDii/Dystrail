//! Consistent action labels with readable costs beneath the primary action.
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct Props {
    pub label: String,
    #[prop_or_default]
    pub detail: String,
    #[prop_or_default]
    pub duration: Option<String>,
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub claimed: bool,
}
#[function_component(ActionButton)]
pub fn action_button(p: &Props) -> Html {
    crate::i18n::use_language();
    let (label, detail) = if p.detail.is_empty() {
        p.label.split_once(" · ").unwrap_or((&p.label, ""))
    } else {
        (p.label.as_str(), p.detail.as_str())
    };
    let detail = p.duration.as_ref().map_or_else(
        || detail.to_owned(),
        |duration| {
            if detail.is_empty() {
                duration.clone()
            } else {
                format!("{detail} · {duration}")
            }
        },
    );
    let description = if p.claimed {
        format!("{detail}. {}", crate::i18n::t("journey.claimed"))
    } else {
        detail.clone()
    };
    html! {<button class="action-button" aria-label={label.to_owned()} aria-description={description} disabled={p.disabled} onclick={p.onclick.clone()}><span class="action-title">{label}</span>if !detail.is_empty(){<small class="action-detail">if p.claimed {<s>{detail}</s>}else{{detail}}</small>}</button>}
}
