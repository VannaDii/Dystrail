use crate::app::phase::build_weather_badge;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::crossing::CrossingPage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_crossing(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        let pending = snapshot.pending_crossing;
        let state_rc = Rc::new(snapshot);
        let config_rc = Rc::new((*state.crossing_config).clone());

        pending.map_or_else(Html::default, |pending| {
            html! {
                <CrossingPage
                    state={state_rc.clone()}
                    config={config_rc}
                    kind={pending.kind}
                    weather={weather_badge}
                    on_choice={handlers.crossing_choice.clone()}
                />
            }
        })
    })
}
