//! Regional scene with readable, synchronized HUD and environmental presentation.
use super::{
    journey_scene::{JourneyScene, SceneStage},
    stats_bar::{self, HudPart, StatsBar},
};
use crate::game::GameState;
use std::rc::Rc;
use yew::prelude::*;
#[derive(Properties, Clone)]
pub struct Props {
    pub state: Rc<GameState>,
    pub title: String,
    #[prop_or_default]
    pub help_text: Option<String>,
    #[prop_or_default]
    pub stage: Option<SceneStage>,
    #[prop_or_default]
    pub moving: bool,
    #[prop_or_default]
    pub local_npc: Option<u8>,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub decision: Html,
}
impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && self.title == other.title
            && self.help_text == other.help_text
            && self.stage == other.stage
            && self.moving == other.moving
            && self.local_npc == other.local_npc
            && self.children == other.children
            && self.decision == other.decision
    }
}
#[function_component(WorldView)]
pub fn world_view(p: &Props) -> Html {
    crate::i18n::use_language();
    let gs = &p.state;
    let weather_cfg = crate::game::WeatherConfig::default_config();
    let stage = p.stage.clone().unwrap_or(SceneStage::Travel(gs.region));
    let hour = u8::try_from(gs.continuity.clock_minutes / 60).unwrap_or(8);
    let subject = match &stage {
        SceneStage::Care => gs.continuity.crew_care.pending.clone(),
        SceneStage::Encounter(_) => gs.continuity.scene_subject.clone(),
        SceneStage::Town if p.local_npc.is_some() => gs.persona_id.clone(),
        _ => None,
    };
    let has_cast = !matches!(
        stage,
        SceneStage::Travel(_) | SceneStage::Setup | SceneStage::Ending(_)
    );
    let cast = if has_cast {
        super::journey_scene::composition::cast_members(
            &gs.party,
            subject.as_deref(),
            gs.day,
            super::journey_scene::asset_name(gs.mode.is_deep(), &stage).unwrap_or("scene"),
            p.local_npc,
        )
    } else {
        Html::default()
    };
    html! {<><div class={classes!("world-view",p.local_npc.is_some().then_some("world-conversation"))} data-region={format!("{:?}",gs.region)} data-day={gs.day.to_string()}>
        <StatsBar part={HudPart::Resources} stats={gs.stats.clone()} receipts={gs.receipts.len()} day={gs.day} region={gs.region} />
        <JourneyScene show_cast={false} local_npc={p.local_npc} region={Some(gs.region)} {subject} road_asset={Some(crate::game::route::road_scene(gs).to_owned())} party={Some(gs.party.clone())} {stage} day={gs.day} {hour} deep={gs.mode.is_deep()} weather={Some(gs.weather_state.today)} moving={p.moving}>
            <StatsBar trip_resources={super::leg_summary::render_hud_resources(gs)} trip_destination={super::leg_summary::render_hud_destination(gs)} pace={Some(gs.pace)} diet={Some(gs.diet)} part={HudPart::Conditions} moving={p.moving} clock_hour={hour} clock_minute={u8::try_from(gs.continuity.clock_minutes % 60).unwrap_or(0)} stats={gs.stats.clone()} receipts={gs.receipts.len()} day={gs.day} region={gs.region} persona_id={gs.persona_id.clone()} exec_order={gs.current_order} policy_readout={gs.current_order.map(|order|stats_bar::policy::readout(gs,order))} weather={Some(crate::app::phase::build_weather_badge(gs,&weather_cfg))} weather_readout={Some(stats_bar::weather::readout(gs,&weather_cfg))} />
            <figcaption class={classes!("scene-footer",has_cast.then_some("has-cast"),p.local_npc.is_some().then_some("has-conversation"))}>
                <div class="scene-caption"><p class="scene-location">{super::route_map::location::location(gs)}</p><h1 id="screen-title" tabindex="-1">{&p.title}if let Some(text)=&p.help_text {<super::context_help::ContextHelp title={p.title.clone()} text={text.clone()} />}</h1></div>
                {cast}
            </figcaption>
            {for p.children.iter()}
        </JourneyScene>
    </div><crate::app::journey_panel::JourneyPanel>{p.decision.clone()}</crate::app::journey_panel::JourneyPanel></>}
}
