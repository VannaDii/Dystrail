use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{CampConfig, EndgameTravelCfg, GameState};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct CampPageProps {
    pub state: Rc<GameState>,
    pub camp_config: Rc<CampConfig>,
    pub endgame_config: Rc<EndgameTravelCfg>,
    pub weather: WeatherBadge,
    pub on_state_change: Callback<GameState>,
    pub on_close: Callback<()>,
}

impl PartialEq for CampPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && Rc::ptr_eq(&self.camp_config, &other.camp_config)
            && Rc::ptr_eq(&self.endgame_config, &other.endgame_config)
            && self.weather == other.weather
    }
}

#[function_component(CampPage)]
pub fn camp_page(props: &CampPageProps) -> Html {
    let stats = props.state.stats.clone();
    let day = props.state.day;
    let region = props.state.region;
    let exec_order = props.state.current_order;
    let persona_id = props.state.persona_id.clone();

    html! {
        <section data-testid="camp-screen">
            <crate::components::ui::stats_bar::StatsBar
                {stats}
                {day}
                {region}
                exec_order={exec_order}
                persona_id={persona_id}
                weather={Some(props.weather.clone())}
            />
            <crate::components::ui::camp_panel::CampPanel
                game_state={props.state.clone()}
                camp_config={props.camp_config.clone()}
                endgame_config={props.endgame_config.clone()}
                on_state_change={props.on_state_change.clone()}
                on_close={props.on_close.clone()}
            />
        </section>
    }
}
