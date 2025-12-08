use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::GameState;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct EncounterPageProps {
    pub state: Rc<GameState>,
    pub weather: WeatherBadge,
    pub on_choice: Callback<usize>,
}

impl PartialEq for EncounterPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state) && self.weather == other.weather
    }
}

#[function_component(EncounterPage)]
pub fn encounter_page(props: &EncounterPageProps) -> Html {
    let stats = props.state.stats.clone();
    let day = props.state.day;
    let region = props.state.region;
    let exec_order = props.state.current_order;
    let persona_id = props.state.persona_id.clone();
    let encounter = props.state.current_encounter.clone();

    encounter.map_or_else(
        || {
            html! { <p class="muted" role="status">{ crate::i18n::t("ui.loading_encounters") }</p> }
        },
        |enc| {
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
                    <crate::components::ui::encounter_card::EncounterCard
                        encounter={enc}
                        on_choice={props.on_choice.clone()}
                    />
                </>
            }
        },
    )
}
