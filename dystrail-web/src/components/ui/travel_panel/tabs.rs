use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Selection {
    pub tab: u8,
    pub expanded: bool,
}
impl Default for Selection {
    fn default() -> Self {
        Self {
            tab: 3,
            expanded: true,
        }
    }
}

pub fn render(detail: &UseStateHandle<Selection>) -> Html {
    let tabs = [
        (3, "journey.report"),
        (2, "journey.conditions"),
        (0, "journey.van"),
        (1, "journey.journal"),
    ];
    html! {<div class="travel-tabs" role="tablist" aria-label={crate::i18n::t("ux.details")}>
        {for tabs.into_iter().enumerate().map(|(index,(n,key))| {
            let selected = detail.expanded && detail.tab == n;
            let onclick = {let detail=detail.clone();Callback::from(move |_|detail.set(Selection {tab:n,expanded:!selected}))};
            let onkeydown = {let detail=detail.clone();Callback::from(move |event: KeyboardEvent| {
                let rtl = crate::i18n::is_rtl();
                let next = match event.key().as_str() {
                    "ArrowRight" => (index + if rtl {3}else{1}) % 4,
                    "ArrowLeft" => (index + if rtl {1}else{3}) % 4,
                    "Home" => 0,
                    "End" => 3,
                    _ => return,
                };
                event.prevent_default();
                detail.set(Selection {tab:tabs[next].0,expanded:true});
                crate::a11y::restore_focus(&format!("travel-tab-{}",tabs[next].0));
            })};
            html! {<button role="tab" id={format!("travel-tab-{n}")} aria-selected={selected.to_string()} aria-expanded={selected.to_string()} aria-controls="travel-detail-panel" tabindex={if detail.tab==n {"0"}else{"-1"}} {onclick} {onkeydown}>{crate::i18n::t(key)}</button>}
        })}
    </div>}
}
