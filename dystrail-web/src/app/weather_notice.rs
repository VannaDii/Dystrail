//! Weather interrupts the road with its actual modifiers and an explicit continuation.
use super::{Phase, state::AppState};
use crate::i18n;
use yew::prelude::*;

pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    let acknowledge = {
        let notice = app.weather_notice.clone();
        Callback::from(move |_| notice.set(false))
    };
    let camp = {
        let app = app.clone();
        Callback::from(move |_| {
            app.weather_notice.set(false);
            app.travel_running.set(false);
            app.phase.set(Phase::Camp);
        })
    };
    html! {<section class="weather-notice" role="alert"><div>
        <h2>{i18n::t("journey.conditions_changed")}</h2>
        {crate::components::ui::travel_panel::weather::render_weather_details(gs)}
        <p>{i18n::t("play2.weather_changed")}</p>
        <div class="controls"><button onclick={acknowledge}>{i18n::t("journey.accept_conditions")}</button><button onclick={camp}>{i18n::t("ux.camp")}</button></div>
    </div></section>}
}
