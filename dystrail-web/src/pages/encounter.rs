use crate::components::ui::journey_scene::SceneStage;
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
    crate::i18n::use_language();
    let encounter = props.state.current_encounter.clone();

    encounter.map_or_else(
        || {
            html! { <p class="muted" role="status">{ crate::i18n::t("ui.loading_encounters") }</p> }
        },
        |mut enc| {
            if let Some(unit) = crate::app::visual_content::encounter_unit(&props.state) {
                enc.id = unit.to_owned(); // presentation clone; engine keeps runtime identity/effects
            }
            html! {
                <>
                    <crate::components::ui::world_view::WorldView state={props.state.clone()} title={crate::i18n::encounter_text(&enc.id,"name",&enc.name)} stage={Some(SceneStage::Encounter(enc.id.clone()))} decision={html! {
                    <crate::components::ui::encounter_card::EncounterCard
                        key={crate::i18n::current_lang()}
                        encounter={enc}
                        stats={props.state.stats.clone()}
                        cash={props.state.budget_cents}
                        receipts={props.state.receipts.len()}
                        receipt_bonus_chance={props.state.receipt_bonus_chance()}
                        on_choice={props.on_choice.clone()}
                    />
                    }} />
                </>
            }
        },
    )
}
