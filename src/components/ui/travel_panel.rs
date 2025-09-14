use crate::i18n;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub on_travel: Callback<()>,
    pub logs: Vec<String>,
}

#[function_component(TravelPanel)]
pub fn travel_panel(p: &Props) -> Html {
    let on_click = {
        let cb = p.on_travel.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! {
        <section class="panel">
            <h2>{ i18n::t("travel.title") }</h2>
            <div class="controls">
                <button onclick={on_click} aria-label={i18n::t("travel.next")} class="retro-btn-primary">
                    { i18n::t("travel.next") }
                </button>
            </div>
            if !p.logs.is_empty() {
                <div class="log" role="log" aria-live="polite">
                    { for p.logs.iter().map(|l| html!{ <p>{l}</p> }) }
                </div>
            }
        </section>
    }
}
