use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{GameState, OtDeluxeRouteDecision, OtDeluxeRoutePrompt};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct RoutePromptPageProps {
    pub state: Rc<GameState>,
    pub prompt: OtDeluxeRoutePrompt,
    pub weather: WeatherBadge,
    pub on_choice: Callback<OtDeluxeRouteDecision>,
}

impl PartialEq for RoutePromptPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && self.prompt == other.prompt
            && self.weather == other.weather
    }
}

#[function_component(RoutePromptPage)]
pub fn route_prompt_page(props: &RoutePromptPageProps) -> Html {
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
            <crate::components::ui::route_prompt_card::RoutePromptCard
                prompt={props.prompt}
                on_choice={props.on_choice.clone()}
            />
        </>
    }
}
