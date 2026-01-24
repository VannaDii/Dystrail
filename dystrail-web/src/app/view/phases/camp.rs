use crate::app::phase::session_from_state;
use crate::app::state::AppState;
use crate::game::state::GameState;
use crate::pages::camp::CampPage;
use std::rc::Rc;
use yew::prelude::*;

pub fn render_camp(state: &AppState) -> Html {
    (*state.session).clone().map_or_else(Html::default, |sess| {
        let snapshot = sess.state().clone();
        let weather_cfg = (*state.weather_config).clone();
        let weather_today = snapshot.weather_state.today;
        let weather_mitigated = weather_cfg
            .mitigation
            .get(&weather_today)
            .is_some_and(|mit| snapshot.inventory.tags.contains(&mit.tag));
        let weather_badge = crate::components::ui::stats_bar::WeatherBadge {
            weather: weather_today,
            mitigated: weather_mitigated,
        };
        let camp_state = Rc::new(snapshot);
        let camp_config_rc = Rc::new((*state.camp_config).clone());
        let endgame_config_rc = Rc::new((*state.endgame_config).clone());
        html! { <CampPage state={camp_state} camp_config={camp_config_rc} endgame_config={endgame_config_rc} weather={weather_badge} on_state_change={{ let session_handle = state.session.clone(); let pending_state = state.pending_state.clone(); let endgame_cfg = (*state.endgame_config).clone(); Callback::from(move |new_state: GameState| { let snapshot = new_state.clone(); let updated = session_from_state(new_state, &endgame_cfg); pending_state.set(Some(snapshot)); session_handle.set(Some(updated)); }) }} on_close={{ let phase_handle = state.phase.clone(); Callback::from(move |()| phase_handle.set(crate::app::phase::Phase::Menu)) }} /> }
    })
}
