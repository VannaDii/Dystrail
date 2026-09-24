//! Deferred presentation of a single already-resolved engine action.
use super::{aftermath::Aftermath, state::AppState};
use crate::{game::JourneySession, game::journal::JournalEntry, i18n};
use std::rc::Rc;
use yew::prelude::*;
pub struct PendingTurn {
    pub session: JourneySession,
    pub report: Aftermath,
    pub logs: Vec<String>,
}

pub fn render_transit(state: &AppState, pending: &Rc<PendingTurn>) -> Html {
    let Some(_) = state.session.as_ref() else {
        return Html::default();
    };
    let complete = {
        let state = state.clone();
        let pending = pending.clone();
        Callback::from(move |()| {
            if !*state.action_lock.borrow() {
                return;
            }
            *state.action_lock.borrow_mut() = false;
            let gs = pending.session.state();
            let next = super::map::transition(
                gs,
                pending.report.next,
                *state.phase == super::Phase::Map && *state.map_automatic,
                *state.travel_speed,
            );
            // A scheduled map stays visible until dismissed. A newly reached decision
            // is retained in the session; the engine refuses to travel past it.
            if super::policy_bulletin::pending(gs).is_some()
                || super::crossing_presentation::pending(gs).is_some()
                || !super::flow::visible()
                || (next != super::Phase::Map
                    && (next != super::Phase::Travel
                        || gs.breakdown.is_some()
                        || gs.stats.supplies <= 2
                        || gs.stats.hp <= 2
                        || gs.stats.sanity <= 2))
            {
                state.travel_running.set(false);
            }
            state.logs.set(pending.logs.clone());
            state.map_automatic.set(next == super::Phase::Map);
            state.session.set(Some(pending.session.clone()));
            state.phase.set(next);
            state.pending_turn.set(None);
        })
    };
    html! {<TravelTransition key={pending.session.state().continuity.journal.len()} on_complete={complete} duration={state.travel_speed.duration()} />}
}

#[derive(Properties, Clone)]
struct Props {
    on_complete: Callback<()>,
    duration: i32,
}
impl PartialEq for Props {
    fn eq(&self, o: &Self) -> bool {
        self.on_complete == o.on_complete && self.duration == o.duration
    }
}
#[function_component(TravelTransition)]
fn travel_transition(p: &Props) -> Html {
    crate::i18n::use_language();
    let latest = use_mut_ref(|| p.on_complete.clone());
    *latest.borrow_mut() = p.on_complete.clone();
    // Keep one cancelable timer. Leaving this screen cannot commit a stale turn.
    {
        let done = latest;
        let duration = p.duration;
        use_effect_with((), move |()| {
            use wasm_bindgen::{JsCast, closure::Closure};
            let callback =
                Closure::wrap(Box::new(move || done.borrow().emit(())) as Box<dyn FnMut()>);
            let window = web_sys::window();
            let timer = window.as_ref().and_then(|win| {
                win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    duration,
                )
                .ok()
            });
            move || {
                if let (Some(win), Some(id)) = (window, timer) {
                    win.clear_timeout_with_handle(id);
                }
                drop(callback);
            }
        });
    }
    Html::default()
}

pub fn render_turn_entries(gs: &crate::game::GameState) -> Html {
    let entries = gs.continuity.turn_entries();
    if entries.is_empty() {
        return Html::default();
    }
    let offset = gs.continuity.journal.len() - entries.len();
    html! {<ol class="turn-receipts" role="log" aria-label={i18n::t("journey.report")} aria-relevant="additions">
        {for entries.iter().enumerate().rev().map(|(index,entry)|render_entry(entry,offset+index))}
    </ol>}
}

fn render_entry(entry: &JournalEntry, index: usize) -> Html {
    let heading = entry_heading(entry);
    html! {<li key={index} class="turn-receipt" data-action={entry.action_kind.clone()}>
        <span class="receipt-mark" aria-hidden="true">{crate::components::ui::journey_icon::render(&entry.action_kind)}</span>
        <div class="receipt-body"><div class="receipt-narrative"><div class="receipt-heading">
            <strong>{heading}</strong>
        </div>
        <dl class="receipt-details">{for entry.details.iter().filter(|(name,_)|crate::app::receipt::narrative_detail(name)).map(|(name,value)|html!{<div><dt>{name}</dt><dd>{value}</dd></div>})}</dl></div>
        {crate::components::ui::stat_card::render_changes(&entry.before, &entry.after, &entry.resources)}
        </div>
    </li>}
}

fn entry_heading(entry: &JournalEntry) -> String {
    let message = i18n::meaningful_message(&entry.message);
    if !message.is_empty() {
        return message;
    }
    let title = entry.title.trim();
    if !title.is_empty() && title != i18n::t("play.last_turn") {
        return title.to_owned();
    }
    i18n::t(match entry.action_kind.as_str() {
        "travel" => "log.traveled",
        "repair" => "play.repairing",
        "camp" => "ux.camp",
        "town" => "journey.arrival",
        "care" => "journey.crew_stop",
        _ => "ux.day_report",
    })
}
