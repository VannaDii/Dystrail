use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{GameState, PacingConfig};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct TravelPageProps {
    pub state: Rc<GameState>,
    pub logs: Vec<String>,
    pub pacing_config: Rc<PacingConfig>,
    pub weather_badge: WeatherBadge,
    pub data_ready: bool,
    pub on_travel: Callback<()>,
    pub on_trade: Callback<()>,
    pub on_hunt: Callback<()>,
    pub on_open_inventory: Callback<()>,
    pub on_open_pace_diet: Callback<()>,
    pub on_open_map: Callback<()>,
}

impl PartialEq for TravelPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && self.logs == other.logs
            && Rc::ptr_eq(&self.pacing_config, &other.pacing_config)
            && self.data_ready == other.data_ready
    }
}

#[function_component(TravelPage)]
pub fn travel_page(props: &TravelPageProps) -> Html {
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
                weather={Some(props.weather_badge.clone())}
            />
            <crate::components::ui::travel_panel::TravelPanel
                on_travel={props.on_travel.clone()}
                on_trade={props.on_trade.clone()}
                on_hunt={props.on_hunt.clone()}
                on_open_inventory={props.on_open_inventory.clone()}
                on_open_pace_diet={props.on_open_pace_diet.clone()}
                on_open_map={props.on_open_map.clone()}
                logs={props.logs.clone()}
                game_state={Some(props.state.clone())}
                pacing_config={props.pacing_config.clone()}
            />
        {
            if props.state.current_encounter.is_some() || props.data_ready {
                Html::default()
            } else {
                html! { <p class="muted" role="status">{ crate::i18n::t("ui.loading_encounters") }</p> }
            }
        }
        </>
    }
}
