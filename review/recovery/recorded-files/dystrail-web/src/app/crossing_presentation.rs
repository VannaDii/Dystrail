//! Acknowledged scenes describe committed telemetry, never reroll or repay it.
use super::{Phase, state::AppState};
use crate::{game::{GameState, journal::CrossingPresentation, state::{CrossingTelemetry, CrossingOutcomeTelemetry, CrossingDetourReason}}, i18n};
use yew::prelude::*;

pub fn family(index: usize, deep: bool) -> &'static str {
    match (index,deep) { (0,_)=>"CROSS-01",(1,false)=>"CROSS-02C",(1,true)=>"CROSS-02D",_=>"CROSS-03" }
}
pub fn capture(before: &GameState, after: &mut GameState) {
    if after.continuity.visual_content.edition != super::visual_content::EDITION { return; }
    for index in before.crossing_events.len()..after.crossing_events.len() {
        if after.continuity.visual_content.crossing_presentations.iter().any(|n|n.event_index==index) { continue; }
        let ordinal=usize::try_from(before.crossings_completed).unwrap_or(0)+index-before.crossing_events.len();
        let unit=super::visual_content::select(after,family(ordinal,after.mode.is_deep()),&format!("crossing/{index}"));
        after.continuity.visual_content.crossing_presentations.push(CrossingPresentation{event_index:index,unit,acknowledged:false});
    }
}
pub fn pending(gs:&GameState)->Option<&CrossingPresentation> {
    gs.continuity.visual_content.crossing_presentations.iter().find(|n| !n.acknowledged && gs.crossing_events.get(n.event_index).is_some() && crate::components::ui::journey_scene::crossing_art::cell(&n.unit).is_some())
}
pub fn is_pending(app:&AppState)->bool {
    app.session.as_ref().is_some_and(|s|pending(s.state()).is_some()) && !matches!(*app.phase,Phase::Boot|Phase::Persona|Phase::Crew|Phase::Outfitting|Phase::Menu)
}
pub fn acknowledge(gs:&mut GameState,index:usize) {
    if let Some(n)=gs.continuity.visual_content.crossing_presentations.iter_mut().find(|n|n.event_index==index) { n.acknowledged=true; }
}
pub fn copy(unit:&str,field:&str)->String { i18n::t(&format!("visual_copy.{unit}.{field}")) }
pub fn message(unit:&str,event:&CrossingTelemetry)->String {
    let field=match event.outcome {
        CrossingOutcomeTelemetry::Passed if event.bribe_success==Some(true)=>"bribe_success",
        CrossingOutcomeTelemetry::Passed=>"passage",
        CrossingOutcomeTelemetry::Detoured if event.detour_reason==Some(CrossingDetourReason::CheckpointDenied) && unit.starts_with("CROSS-01-")=>"refused_passage",
        CrossingOutcomeTelemetry::Detoured=>"diversion",
        CrossingOutcomeTelemetry::Failed=>"terminal_failure",
    };
    let mut lines=vec![copy(unit,"setup")];
    if event.bribe_success==Some(false) { lines.push(copy(unit,"outcomes.bribe_failure")); }
    lines.push(copy(unit,&format!("outcomes.{field}")));lines.join(" ")
}
pub fn render(app:&AppState)->Html {
    let Some(gs)=app.session.as_ref().map(|s|s.state()) else{return Html::default()};
    let Some(notice)=pending(gs) else{return Html::default()};
    let event=&gs.crossing_events[notice.event_index];
    let unit=&notice.unit;
    let stage=crate::components::ui::journey_scene::SceneStage::Crossing{unit:unit.clone(),outcome:event.outcome};
    let on_continue={let app=app.clone();let index=notice.event_index;Callback::from(move |_|{
        let Some(mut session)=(*app.session).clone() else{return};
        // Ignore stale/double activation. Other pending presentations and the
        // actual phase/aftermath survive this acknowledgment unchanged.
        if pending(session.state()).is_none_or(|n|n.event_index!=index){return;}
        session.with_state_mut(|gs|acknowledge(gs,index));
        app.travel_running.set(false);
        app.session.set(Some(session));
    })};
    html!{<crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={copy(unit,"title")} stage={Some(stage)} decision={html!{
        <section class="crossing-outcome" data-event-index={notice.event_index.to_string()}>
            <p class="crossing-message">{message(unit,event)}</p>
            <dl class="crossing-receipt">
                if event.bribe_cost_cents>0 {<div data-crossing-cost="cash"><dt>{i18n::t("journey.cash")}</dt><dd>{format!("−${:.2}",event.bribe_cost_cents as f64 /100.0)}</dd></div>}
                if let Some(hours)=event.detour_hours {<div data-crossing-cost="time"><dt>{i18n::t("crossing.detour")}</dt><dd>{format!("{hours} h")}</dd></div>}
            </dl>
            {crate::components::ui::satire_context::unit_hook(unit,unit.trim_end_matches("-A"))}
            <div class="outcome-actions"><button id="crossing-continue" class="btn btn-primary" onclick={on_continue}>{i18n::t("ui.continue")}</button></div>
        </section>
    }}/>} 
}
