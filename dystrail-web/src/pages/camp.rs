use crate::components::ui::journey_scene::SceneStage;
use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{CampConfig, GameState};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct CampPageProps {
    pub state: Rc<GameState>,
    pub camp_config: Rc<CampConfig>,
    pub weather: WeatherBadge,
    pub on_state_change: Callback<(GameState, String)>,
    pub on_close: Callback<()>,
    #[prop_or_default]
    pub gathering: Html,
}

impl PartialEq for CampPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && Rc::ptr_eq(&self.camp_config, &other.camp_config)
            && self.weather == other.weather
            && self.gathering == other.gathering
    }
}

#[function_component(CampPage)]
pub fn camp_page(props: &CampPageProps) -> Html {
    crate::i18n::use_language();
    html! {
        <>
            <crate::components::ui::world_view::WorldView state={props.state.clone()} title={crate::i18n::t("ux.camp")} help_text={Some(crate::i18n::t("journey.camp_forecast"))} stage={Some(SceneStage::Camp)} />
            <crate::components::ui::camp_panel::CampPanel
                game_state={props.state.clone()}
                camp_config={props.camp_config.clone()}
                on_state_change={props.on_state_change.clone()}
                on_close={props.on_close.clone()}
                gathering={props.gathering.clone()}
            />
        </>
    }
}
