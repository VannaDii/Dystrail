use crate::app::phase::build_weather_badge;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::game::MechanicalPolicyId;
use crate::pages::otdeluxe_store::OtDeluxeStorePage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_store(state: &AppState, handlers: &AppHandlers) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_badge = build_weather_badge(&snapshot, &state.weather_config);
        let state_rc = Rc::new(snapshot);
        if state_rc.mechanical_policy != MechanicalPolicyId::OtDeluxe90s {
            return Html::default();
        }
        if state_rc.ot_deluxe.store.pending_node.is_none() {
            return Html::default();
        }
        html! {
            <OtDeluxeStorePage
                state={state_rc}
                weather={weather_badge}
                on_purchase={handlers.store_purchase.clone()}
                on_leave={handlers.store_leave.clone()}
            />
        }
    })
}
