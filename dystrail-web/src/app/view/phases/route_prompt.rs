use crate::app::phase::build_weather_badge;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::route_prompt::RoutePromptPage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_route_prompt(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        let pending = snapshot.ot_deluxe.route.pending_prompt;
        let state_rc = Rc::new(snapshot);

        pending.map_or_else(Html::default, |prompt| html! { <RoutePromptPage state={state_rc.clone()} prompt={prompt} weather={weather_badge} on_choice={handlers.route_prompt_choice.clone()} /> })
    })
}
