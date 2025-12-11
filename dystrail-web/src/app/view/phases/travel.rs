use crate::app::phase::build_weather_badge;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::travel::TravelPage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_travel(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        let state_rc = Rc::new(snapshot);
        let pacing_config_rc = Rc::new((*state.pacing_config).clone());
        html! {
            <TravelPage
                state={state_rc}
                logs={(*state.logs).clone()}
                pacing_config={pacing_config_rc}
                weather_badge={weather_badge}
                data_ready={state.data_ready()}
                on_travel={handlers.travel.clone()}
                on_pace_change={handlers.pace_change.clone()}
                on_diet_change={handlers.diet_change.clone()}
            />
        }
    })
}
