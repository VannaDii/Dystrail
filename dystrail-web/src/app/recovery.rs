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
    last_turn: Option<Aftermath>,
    #[serde(default)]
    weather_notice: bool,
    #[serde(default)]
    town_open: bool,
    #[serde(default)]
    travel_speed: super::flow::TravelSpeed,
}
fn restore(app: &AppState, saved: Checkpoint) {
    app.pending_state.set(saved.pending);
    if let Some(gs) = saved.state {
        app.run_seed.set(gs.seed);
        app.session.set(Some(session_from_state(
            gs.rehydrate((*app.data).clone()),
            &app.endgame_config,
        )));
    }
    app.code.set(saved.code.into());
    app.phase.set(saved.phase);
    app.logs.set(saved.logs);
    app.aftermath.set(saved.aftermath);
    app.last_turn.set(saved.last_turn);
    app.weather_notice.set(saved.weather_notice);
    app.town_open.set(saved.town_open);
    app.travel_speed.set(saved.travel_speed);
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
                let changed_weather = p.session.state().weather_state.today
                    != crate::game::weather::Weather::Clear
                    && app.session.as_ref().is_some_and(|s| {
                        s.state().weather_state.today != p.session.state().weather_state.today
                    });
                if changed_weather {
                    p.report.next
                } else {
                    super::map::after_transit(p.session.state(), p.report.next)
                }
            }),
            code: app.code.to_string(),
            logs: pending.map_or_else(|| (*app.logs).clone(), |p| p.logs.clone()),
            aftermath: if pending.is_some() {
                None
            } else {
                (*app.aftermath).clone()
            },
            last_turn: pending.map_or_else(|| (*app.last_turn).clone(), |p| Some(p.report.clone())),
            weather_notice: pending.map_or(*app.weather_notice, |p| {
                p.session.state().weather_state.today != crate::game::weather::Weather::Clear
                    && app.session.as_ref().is_some_and(|s| {
                        s.state().weather_state.today != p.session.state().weather_state.today
                    })
            }),
            town_open: *app.town_open,
            travel_speed: *app.travel_speed,
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
