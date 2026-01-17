use crate::app::phase::build_weather_badge;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::game::MechanicalPolicyId;
use crate::pages::crossing::CrossingPage;
use crate::pages::otdeluxe_crossing::OtDeluxeCrossingPage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_crossing(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        let state_rc = Rc::new(snapshot);
        if state_rc.mechanical_policy == MechanicalPolicyId::OtDeluxe90s {
            if state_rc.ot_deluxe.crossing.choice_pending {
                return html! {
                    <OtDeluxeCrossingPage
                        state={state_rc}
                        weather={weather_badge}
                        on_choice={handlers.otdeluxe_crossing_choice.clone()}
                    />
                };
            }
            return Html::default();
        }

        let pending = state_rc.pending_crossing;
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
