use crate::i18n;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub title: AttrValue,
    pub summary: AttrValue,
    pub seed_code: AttrValue,
    pub on_share: Callback<()>,
}

#[function_component(ResultScreen)]
pub fn result_screen(p: &Props) -> Html {
    let on_share = {
        let cb = p.on_share.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! {
        <section class="panel result-panel" role="region" aria-label={i18n::t("result.title_aria") }>
            <h1 class="result-title">{ p.title.clone() }</h1>
            <div class="result-summary">
                <p>{ p.summary.clone() }</p>
            </div>
            <div class="seed-display">
                <div class="stat-label">{ i18n::t("game.seed_label") }</div>
                <div class="stat-value">{ p.seed_code.clone() }</div>
            </div>
            <div class="controls">
                <button class="retro-btn-primary" onclick={on_share}>{ i18n::t("share.run") }</button>
            </div>
        </section>
    }
}
