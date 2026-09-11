use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{DietId, GameState, PaceId, PacingConfig};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct TravelPageProps {
    pub state: Rc<GameState>,
    #[prop_or_default]
    pub receipt: Html,
    pub moving: bool,
    pub progress: f32,
    pub duration: i32,
    pub transit: Html,
    pub activities: Html,
    pub repair: Html,
    #[prop_or_default]
    pub controls: Html,
    pub logs: Vec<String>,
    pub pacing_config: Rc<PacingConfig>,
    pub weather_badge: WeatherBadge,
    pub data_ready: bool,
    pub on_travel: Callback<()>,
    pub on_pace_change: Callback<PaceId>,
    pub on_diet_change: Callback<DietId>,
}

impl PartialEq for TravelPageProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && self.moving == other.moving
            && self.transit == other.transit
            && self.receipt == other.receipt
            && self.on_travel == other.on_travel
            && self.logs == other.logs
            && Rc::ptr_eq(&self.pacing_config, &other.pacing_config)
            && self.data_ready == other.data_ready
    }
}

#[function_component(TravelPage)]
pub fn travel_page(props: &TravelPageProps) -> Html {
    let total = crate::game::route::for_state(&props.state)
        .map_or(props.state.trail_distance, |r| r.total_miles);
    let percent = props.progress / total.max(1.0) * 100.0;
    html! { <div class="travel-screen" style={format!("--travel-duration:{}ms",props.duration)}>
        <crate::components::ui::world_view::WorldView state={props.state.clone()} title={crate::i18n::t(if props.state.breakdown.is_some(){"play.repairing"}else{"play.journey"})} stage={props.state.breakdown.as_ref().map(|_|crate::components::ui::journey_scene::SceneStage::Breakdown)} moving={props.moving && props.state.breakdown.is_none()} >
            <div class="route-progress" role="progressbar" aria-label={crate::i18n::t("play.travel_progress")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={format!("{percent:.1}")}><span style={format!("width:{percent:.3}%")}></span></div>
        </crate::components::ui::world_view::WorldView>
        <div class="travel-toolbar">
        if props.moving {<div class="transit-caption"><p role="status">{crate::i18n::t("journey.rolling")}</p><button onclick={{let on=props.on_travel.clone();Callback::from(move |_|on.emit(()))}}>{crate::i18n::t("journey.pause")}</button></div>}
        else {{props.controls.clone()}}
        </div>
        if props.state.breakdown.is_some() {{props.repair.clone()}}
        else if !props.moving {
            <crate::components::ui::travel_panel::TravelPanel receipt={props.receipt.clone()} on_travel={props.on_travel.clone()} logs={props.logs.clone()} game_state={Some(props.state.clone())} pacing_config={props.pacing_config.clone()} on_pace_change={props.on_pace_change.clone()} on_diet_change={props.on_diet_change.clone()} />
            {props.activities.clone()}
        }
        {props.transit.clone()}
    </div> }
}
