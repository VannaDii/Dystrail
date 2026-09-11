//! Regional scene with readable, synchronized HUD and environmental presentation.
use super::{
    journey_scene::{JourneyScene, SceneStage},
    stats_bar::{StatsBar, WeatherBadge},
};
use crate::game::GameState;
use std::rc::Rc;
use yew::prelude::*;
#[derive(Properties, Clone)]
pub struct Props {
    pub state: Rc<GameState>,
    pub title: String,
    #[prop_or_default]
    pub stage: Option<SceneStage>,
    #[prop_or_default]
    pub moving: bool,
    #[prop_or_default]
    pub local_npc: Option<u8>,
    #[prop_or_default]
    pub children: Children,
}
impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && self.title == other.title
            && self.stage == other.stage
            && self.moving == other.moving
            && self.local_npc == other.local_npc
            && self.children == other.children
    }
}
#[function_component(WorldView)]
pub fn world_view(p: &Props) -> Html {
    let gs = &p.state;
    let stage = p.stage.clone().unwrap_or(SceneStage::Travel(gs.region));
    let hour = u8::try_from(gs.continuity.clock_minutes / 60).unwrap_or(8);
    let subject = match &stage {
        SceneStage::Care => gs.continuity.crew_care.pending.clone(),
        SceneStage::Encounter(_) => gs.continuity.scene_subject.clone(),
        SceneStage::Town if p.local_npc.is_some() => gs.persona_id.clone(),
        _ => None,
    };
    html! {<div class={classes!("world-view",p.local_npc.is_some().then_some("world-conversation"))} data-region={format!("{:?}",gs.region)} data-day={gs.day.to_string()}>
        <JourneyScene local_npc={p.local_npc} region={Some(gs.region)} {subject} road_asset={Some(crate::game::route::road_scene(gs).to_owned())} party={Some(gs.party.clone())} {stage} day={gs.day} {hour} deep={gs.mode.is_deep()} weather={Some(gs.weather_state.today)} moving={p.moving}>
            <StatsBar moving={p.moving} clock_hour={hour} clock_minute={u8::try_from(gs.continuity.clock_minutes % 60).unwrap_or(0)} stats={gs.stats.clone()} day={gs.day} region={gs.region} persona_id={gs.persona_id.clone()} exec_order={gs.current_order} weather={Some(WeatherBadge{weather:gs.weather_state.today,mitigated:false})} />
            <figcaption class="scene-caption"><p class="scene-location">{super::route_map::location::location(gs)}</p><h1 id="screen-title" tabindex="-1">{&p.title}</h1></figcaption>
            <div class="scene-status">{super::leg_summary::render_scene(gs,p.moving)}</div>
            {for p.children.iter()}
        </JourneyScene>
    </div>}
}
