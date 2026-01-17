use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::GameState;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct OtDeluxeCrossingPageProps {
    pub state: Rc<GameState>,
    pub weather: WeatherBadge,
    pub on_choice: Callback<u8>,
}

impl PartialEq for OtDeluxeCrossingPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state) && self.weather == other.weather
    }
}

#[function_component(OtDeluxeCrossingPage)]
pub fn otdeluxe_crossing_page(props: &OtDeluxeCrossingPageProps) -> Html {
    let stats = props.state.stats.clone();
    let day = props.state.day;
    let region = props.state.region;
    let exec_order = props.state.current_order;
    let persona_id = props.state.persona_id.clone();

    html! {
        <>
            <crate::components::ui::stats_bar::StatsBar
                {stats}
                {day}
                {region}
                exec_order={exec_order}
                persona_id={persona_id}
                weather={Some(props.weather.clone())}
            />
            <crate::components::ui::otdeluxe_crossing_card::OtDeluxeCrossingCard
                game_state={props.state.clone()}
                on_choice={props.on_choice.clone()}
            />
        </>
    }
}
