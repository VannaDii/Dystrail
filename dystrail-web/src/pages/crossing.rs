use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{CrossingConfig, CrossingKind, GameState};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct CrossingPageProps {
    pub state: Rc<GameState>,
    pub config: Rc<CrossingConfig>,
    pub kind: CrossingKind,
    pub weather: WeatherBadge,
    pub on_choice: Callback<u8>,
}

impl PartialEq for CrossingPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && Rc::ptr_eq(&self.config, &other.config)
            && self.kind == other.kind
            && self.weather == other.weather
    }
}

#[function_component(CrossingPage)]
pub fn crossing_page(props: &CrossingPageProps) -> Html {
    let stats = props.state.stats.clone();
    let day = props.state.day;
    let region = props.state.region;
    let exec_order = props.state.current_order;
    let persona_id = props.state.persona_id.clone();

    html! {
        <section data-testid="crossing-screen">
            <crate::components::ui::stats_bar::StatsBar
                {stats}
                {day}
                {region}
                exec_order={exec_order}
                persona_id={persona_id}
                weather={Some(props.weather.clone())}
            />
            <crate::components::ui::crossing_card::CrossingCard
                game_state={props.state.clone()}
                config={props.config.clone()}
                kind={props.kind}
                on_choice={props.on_choice.clone()}
            />
        </section>
    }
}
