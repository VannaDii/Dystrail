use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::components::ui::journey_scene::SceneStage;
use crate::game::state::{DietId, PaceId};
use yew::prelude::*;

pub fn build_travel(state: &AppState) -> Callback<()> {
    let state = state.clone();
    Callback::from(move |()| {
        if *state.action_lock.borrow() || state.aftermath.is_some() {
            return;
        }
        let Some(mut sess) = (*state.session).clone() else {
            return;
        };
        if sess.state().current_encounter.is_some()
            || sess.state().continuity.crew_care.pending.is_some()
            || sess.state().continuity.abandoned
            || sess.state().ending.is_some()
            || sess.state().continuity.route_services.stop.is_some()
            || sess.state().breakdown.is_some()
        {
            return;
        }
        *state.action_lock.borrow_mut() = true;
        let before = sess.state().clone();
        let scene = SceneStage::Travel(before.region);
        if !sess.state().day_state.lifecycle.day_initialized {
            sess.with_state_mut(|gs| gs.apply_pace_and_diet(&state.pacing_config));
        }
        let outcome = sess.tick_day();
        if sess.state().day > before.day
            || sess.state().miles_traveled_actual > before.miles_traveled_actual
        {
            sess.with_state_mut(|gs| gs.update_route_services(before.miles_traveled_actual));
        }
        sess.with_state_mut(|gs| {
            gs.check_crew(before.day);
            gs.continuity.scene_subject = gs.current_encounter.as_ref().and_then(|enc| {
                crate::components::ui::journey_scene::composition::subject(
                    &gs.party, None, gs.day, &enc.id,
                )
                .map(|m| m.persona.clone())
            });
        });
        let gs = sess.state();
        let mut events: Vec<String> = gs
            .logs
            .iter()
            .skip(before.logs.len())
            .map(|line| crate::i18n::log_message(line))
            .collect();
        let final_line = crate::i18n::log_message(&outcome.log_key);
        if !events.contains(&final_line) {
            events.push(final_line);
        }
        let mut logs = (*state.logs).clone();
        logs.extend(events.clone());
        let next = if outcome.ended {
            Phase::Result
        } else {
            crate::app::aftermath::next_phase(gs)
        };
        let mut details = crate::app::receipt::resource_details(&before, gs);
        weather_details(&before, gs, &state.weather_config, &mut details);
        let mut report = crate::app::aftermath::Aftermath {
            title: crate::i18n::t(if before.breakdown.is_some() {
                "play.repair_receipt"
            } else {
                "play.last_turn"
            }),
            message: events.join(" "),
            scene,
            before: before.stats.clone(),
            after: gs.stats.clone(),
            next,
            details,
        };
        sess.with_state_mut(|gs| crate::app::history::record(&before, gs, &mut report, 120));
        state
            .pending_turn
            .set(Some(std::rc::Rc::new(crate::app::turn::PendingTurn {
                session: sess,
                report,
                logs,
            })));
    })
}

pub fn build_pace_change(state: &AppState) -> Callback<PaceId> {
    let session_handle = state.session.clone();
    let lock = state.action_lock.clone();
    Callback::from(move |new_pace: PaceId| {
        if *lock.borrow() {
            return;
        }
        if let Some(mut sess) = (*session_handle).clone() {
            if sess.state().day_state.lifecycle.day_initialized {
                return;
            }
            sess.with_state_mut(|gs| gs.pace = new_pace);
            session_handle.set(Some(sess));
        }
    })
}

pub fn build_diet_change(state: &AppState) -> Callback<DietId> {
    let session_handle = state.session.clone();
    let lock = state.action_lock.clone();
    Callback::from(move |new_diet: DietId| {
        if *lock.borrow() {
            return;
        }
        if let Some(mut sess) = (*session_handle).clone() {
            if sess.state().day_state.lifecycle.day_initialized {
                return;
            }
            sess.with_state_mut(|gs| gs.diet = new_diet);
            session_handle.set(Some(sess));
        }
    })
}

pub fn build_encounter_choice(state: &AppState) -> Callback<usize> {
    let state = state.clone();
    Callback::from(move |idx: usize| {
        if *state.action_lock.borrow() || state.aftermath.is_some() {
            return;
        }
        let Some(mut sess) = (*state.session).clone() else {
            return;
        };
        let Some(encounter) = sess.state().current_encounter.clone() else {
            return;
        };
        let Some(choice) = encounter.choices.get(idx) else {
            return;
        };
        if !choice.effects.affordable(
            &sess.state().stats,
            sess.state().budget_cents,
            sess.state().receipts.len(),
        ) {
            return;
        }
        *state.action_lock.borrow_mut() = true;
        let before = sess.state().clone();
        let message = choice
            .effects
            .log
            .clone()
            .unwrap_or_else(|| choice.label.clone());
        sess.with_state_mut(|gs| gs.apply_choice(idx));
        let message = crate::i18n::encounter_text(&encounter.id, &format!("log_{idx}"), &message);
        let mut logs = (*state.logs).clone();
        logs.push(message.clone());
        let mut report = crate::app::aftermath::Aftermath {
            title: crate::i18n::encounter_text(&encounter.id, "name", &encounter.name),
            message: before
                .continuity
                .scene_subject
                .as_ref()
                .and_then(|id| before.party.members.iter().find(|m| &m.persona == id))
                .map_or_else(
                    || message.clone(),
                    |member| format!("{}: {message}", member.name),
                ),
            scene: SceneStage::Encounter(encounter.id),
            details: crate::app::receipt::resource_details(&before, sess.state()),
            before: before.stats.clone(),
            after: sess.state().stats.clone(),
            next: crate::app::aftermath::next_phase(sess.state()),
        };
        sess.with_state_mut(|gs| crate::app::history::record(&before, gs, &mut report, 30));
        crate::app::history::publish(&state, report, true);
        state.logs.set(logs);
        state.session.set(Some(sess));
    })
}

fn weather_details(
    before: &crate::game::GameState,
    gs: &crate::game::GameState,
    config: &crate::game::WeatherConfig,
    details: &mut Vec<(String, String)>,
) {
    if !before.day_state.lifecycle.day_initialized
        && gs.weather_state.today != crate::game::weather::Weather::Clear
    {
        let weather = gs.weather_state.today;
        if let Some(effect) = config.effects.get(&weather) {
            let mitigation = config
                .mitigation
                .get(&weather)
                .filter(|m| gs.inventory.tags.contains(&m.tag));
            details.push((
                format!(
                    "{} · {}",
                    crate::i18n::t("play2.weather_contribution"),
                    crate::i18n::t(weather.i18n_key())
                ),
                format!(
                    "{} {:+} · {} {:+} · {} {:+.0}%",
                    crate::i18n::t("ux.supplies"),
                    effect.supplies,
                    crate::i18n::t("ux.sanity"),
                    mitigation.and_then(|m| m.sanity).unwrap_or(effect.sanity),
                    crate::i18n::t("play.distance"),
                    (effect.travel_mult - 1.0) * 100.0
                ),
            ));
        }
    }
}
