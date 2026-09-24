//! Narrative receipts for committed crossing telemetry, with no further simulation.
use super::{Phase, state::AppState};
use crate::{
    game::{
        GameState,
        journal::CrossingPresentation,
        state::{CrossingDetourReason, CrossingOutcomeTelemetry},
    },
    i18n,
};
use yew::prelude::*;

fn family(index: usize, deep: bool) -> &'static str {
    match (index, deep) {
        (0, _) => "CROSS-01",
        (1, false) => "CROSS-02C",
        (1, true) => "CROSS-02D",
        _ => "CROSS-03",
    }
}
pub fn capture(before: &GameState, after: &mut GameState) {
    if after.continuity.visual_content.edition != super::visual_content::EDITION {
        return;
    }
    for index in before.crossing_events.len()..after.crossing_events.len() {
        if after
            .continuity
            .visual_content
            .crossing_presentations
            .iter()
            .any(|n| n.event_index == index)
        {
            continue;
        }
        let ordinal = usize::try_from(before.crossings_completed).unwrap_or_default() + index
            - before.crossing_events.len();
        let unit = super::visual_content::service_unit(
            after,
            family(ordinal, after.mode.is_deep()),
            "crossing",
            u32::try_from(index).unwrap_or_default(),
        );
        let permit_receipt = after.crossing_events[index].permit_used
            && after.receipts.len() < before.receipts.len();
        after
            .continuity
            .visual_content
            .crossing_presentations
            .push(CrossingPresentation {
                event_index: index,
                unit,
                permit_receipt,
                acknowledged: false,
            });
    }
}
pub fn pending(gs: &GameState) -> Option<&CrossingPresentation> {
    gs.continuity
        .visual_content
        .crossing_presentations
        .iter()
        .find(|n| !n.acknowledged && gs.crossing_events.get(n.event_index).is_some())
}
pub fn is_pending(app: &AppState) -> bool {
    app.session
        .as_ref()
        .is_some_and(|s| pending(s.state()).is_some())
        && !matches!(
            *app.phase,
            Phase::Boot | Phase::Persona | Phase::Crew | Phase::Outfitting | Phase::Menu
        )
}
fn copy(unit: &str, field: &str) -> String {
    i18n::encounter_text(unit, field, "")
}
fn message(gs: &GameState, n: &CrossingPresentation) -> String {
    let event = &gs.crossing_events[n.event_index];
    let field = match event.outcome {
        CrossingOutcomeTelemetry::Passed if event.permit_used => {
            if n.permit_receipt {
                "permit_receipt"
            } else {
                "permit_tag"
            }
        }
        CrossingOutcomeTelemetry::Passed if event.bribe_success == Some(true) => "bribe_success",
        CrossingOutcomeTelemetry::Passed => "passage",
        CrossingOutcomeTelemetry::Detoured
            if event.detour_reason == Some(CrossingDetourReason::CheckpointDenied)
                && n.unit.starts_with("CROSS-01-") =>
        {
            "refused_passage"
        }
        CrossingOutcomeTelemetry::Detoured => "diversion",
        CrossingOutcomeTelemetry::Failed => "terminal_failure",
    };
    let mut lines = vec![copy(&n.unit, "desc")];
    if event.bribe_success == Some(false) {
        lines.push(copy(&n.unit, "bribe_failure"));
    }
    lines.push(copy(&n.unit, field));
    lines.join(" ")
}
pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(|s| s.state()) else {
        return Html::default();
    };
    let Some(n) = pending(gs) else {
        return Html::default();
    };
    let on_continue = {
        let app = app.clone();
        let index = n.event_index;
        Callback::from(move |_| {
            let Some(mut session) = (*app.session).clone() else {
                return;
            };
            if pending(session.state()).is_none_or(|n| n.event_index != index) {
                return;
            }
            session.with_state_mut(|gs| {
                if let Some(n) = gs
                    .continuity
                    .visual_content
                    .crossing_presentations
                    .iter_mut()
                    .find(|n| n.event_index == index)
                {
                    n.acknowledged = true;
                }
            });
            app.travel_running.set(false);
            app.session.set(Some(session));
        })
    };
    use crate::components::ui::journey_scene::{SceneStage, crossing_art};
    let event = &gs.crossing_events[n.event_index];
    let stage = if crossing_art::supported(&n.unit) && event.outcome != CrossingOutcomeTelemetry::Failed {
        SceneStage::Crossing { unit: n.unit.clone(), passed: event.outcome == CrossingOutcomeTelemetry::Passed }
    } else { SceneStage::Travel(gs.region) };
    html! {<crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={copy(&n.unit,"name")} stage={Some(stage)} decision={html!{
        <section class="crossing-outcome outcome-screen" data-event-index={n.event_index.to_string()}>
            <div class="aftermath-panel"><p class="crossing-message">{message(gs,n)}</p></div>
            <div class="outcome-actions"><button id="crossing-continue" class="btn btn-primary" onclick={on_continue}>{i18n::t("ui.continue")}</button></div>
        </section>
    }}/>}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{CrossingKind, state::CrossingTelemetry};
    #[test]
    fn committed_crossings_are_captured_once_without_reapplying_effects() {
        for ordinal in 0..3 {
            for deep in [false, true] {
                let mut before = GameState::default();
                before.continuity.visual_content.edition = 1;
                if deep {
                    before.mode = crate::game::GameMode::Deep;
                }
                before.crossings_completed = ordinal;
                before.receipts.push("permit".into());
                let mut after = before.clone();
                after.receipts.clear();
                after.crossings_completed += 1;
                after.crossing_events.push(CrossingTelemetry {
                    day: after.day,
                    region: after.region,
                    season: after.season,
                    kind: CrossingKind::Checkpoint,
                    permit_used: true,
                    bribe_attempted: false,
                    bribe_success: None,
                    bribe_cost_cents: 0,
                    bribe_chance: None,
                    bribe_roll: None,
                    detour_reason: None,
                    detour_taken: false,
                    detour_hours: None,
                    detour_base_supplies_delta: None,
                    detour_extra_supplies_loss: None,
                    terminal_threshold: 0.0,
                    terminal_roll: None,
                    outcome: CrossingOutcomeTelemetry::Passed,
                });
                let mechanics = serde_json::to_value(&after).unwrap();
                capture(&before, &mut after);
                capture(&before, &mut after);
                assert_eq!(
                    after.continuity.visual_content.crossing_presentations.len(),
                    1
                );
                let mut restored: GameState =
                    serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
                let n = pending(&restored).unwrap();
                assert!(n.unit.starts_with(family(ordinal as usize, deep)));
                assert!(n.permit_receipt);
                let unit = n.unit.clone();
                assert!(message(&restored, n).ends_with(&copy(&unit, "permit_receipt")));
                restored.continuity.visual_content.crossing_presentations[0].acknowledged = true;
                assert!(pending(&restored).is_none());
                let mut result = serde_json::to_value(&restored).unwrap();
                result["visual_content"] = mechanics["visual_content"].clone();
                assert_eq!(result, mechanics);
            }
        }
    }
}
