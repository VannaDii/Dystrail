//! Continuous local recovery, separate from the player's manual save slot.
use super::{
    aftermath::Aftermath,
    phase::{Phase, session_from_state},
    state::AppState,
};
use crate::game::GameState;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
const KEY: &str = "dystrail.autosave.v1";
#[derive(Serialize, Deserialize)]
struct Checkpoint {
    state: Option<GameState>,
    pending: Option<GameState>,
    phase: Phase,
    code: String,
    logs: Vec<String>,
    aftermath: Option<Aftermath>,
    #[serde(default)]
    town_open: bool,
    #[serde(default)]
    travel_speed: super::flow::TravelSpeed,
    #[serde(default)]
    map_automatic: bool,
    #[serde(default)]
    map_return: Option<Phase>,
    #[serde(default)]
    journey_detail: crate::components::ui::travel_panel::tabs::Selection,
}
fn restore(app: &AppState, saved: Checkpoint) {
    let phase = if saved
        .state
        .as_ref()
        .is_some_and(|gs| gs.boss.outcome.attempted)
    {
        super::aftermath::next_phase(saved.state.as_ref().expect("state checked"))
    } else if saved.phase == Phase::Camp
        && saved
            .state
            .as_ref()
            .is_some_and(|gs| gs.breakdown.is_some())
    {
        Phase::Travel
    } else {
        saved.phase
    };
    app.pending_state.set(saved.pending);
    if let Some(gs) = saved.state {
        app.run_seed.set(gs.seed);
        app.session.set(Some(session_from_state(
            gs.rehydrate((*app.data).clone()),
            &app.endgame_config,
        )));
    }
    app.code.set(saved.code.into());
    app.phase.set(phase);
    app.logs.set(saved.logs);
    app.aftermath.set(saved.aftermath);
    app.town_open.set(saved.town_open);
    app.travel_speed.set(saved.travel_speed);
    app.map_automatic.set(saved.map_automatic);
    app.map_return.set(saved.map_return);
    app.journey_detail.set(saved.journey_detail);
}
#[hook]
pub fn use_recovery(app: &AppState) {
    {
        let app = app.clone();
        use_effect_with(*app.boot_ready, move |ready| {
            if *ready && !*app.recovery_ready {
                if let Some(storage) =
                    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
                    && let Ok(Some(text)) = storage.get_item(KEY)
                    && let Ok(saved) = serde_json::from_str::<Checkpoint>(&text)
                {
                    restore(&app, saved);
                }
                app.recovery_ready.set(true);
            }
        });
    }
    let snapshot = if *app.recovery_ready {
        let pending = app.pending_turn.as_ref();
        serde_json::to_string(&Checkpoint {
            state: pending
                .map(|p| p.session.state().clone())
                .or_else(|| app.session.as_ref().map(|s| s.state().clone())),
            pending: (*app.pending_state).clone(),
            phase: pending.map_or(*app.phase, |p| {
                super::map::transition(
                    p.session.state(),
                    p.report.next,
                    *app.phase == Phase::Map && *app.map_automatic,
                    *app.travel_speed,
                )
            }),
            code: app.code.to_string(),
            logs: pending.map_or_else(|| (*app.logs).clone(), |p| p.logs.clone()),
            aftermath: if pending.is_some() {
                None
            } else {
                (*app.aftermath).clone()
            },
            town_open: *app.town_open,
            travel_speed: *app.travel_speed,
            map_automatic: pending.map_or(*app.map_automatic, |p| {
                super::map::transition(
                    p.session.state(),
                    p.report.next,
                    *app.phase == Phase::Map && *app.map_automatic,
                    *app.travel_speed,
                ) == Phase::Map
            }),
            map_return: *app.map_return,
            journey_detail: *app.journey_detail,
        })
        .ok()
    } else {
        None
    };
    let status = app.save_status.clone();
    use_effect_with(snapshot, move |text| {
        if let Some(text) = text {
            let saved = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .is_some_and(|storage| storage.set_item(KEY, text).is_ok());
            if !saved {
                status.set(crate::i18n::t("play2.autosave_failed"));
            }
        }
    });
}
