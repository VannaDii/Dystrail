use crate::app::phase::build_weather_badge;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::encounter::EncounterPage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_encounter(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        html! {
            <EncounterPage
                state={Rc::new(snapshot)}
                weather={weather_badge}
                on_choice={handlers.encounter_choice.clone()}
            />
        }
    })
}
