use super::BossPageProps;
use crate::{app::view::handlers::HearingCommand, game::boss::HearingPhase};
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

pub(super) const fn duration(phase: HearingPhase, fast: bool, reduced: bool) -> Option<i32> {
    let millis = match phase {
        HearingPhase::RoundRolling(_) => 2600,
        HearingPhase::Committee(_) => 1200,
        HearingPhase::CommitteeResult(_) => 1800,
        HearingPhase::VoteRolling => 3000,
        _ => return None,
    };
    Some(if reduced {
        120
    } else if fast {
        400
    } else {
        millis
    })
}

#[hook]
pub(super) fn use_presentation(p: &BossPageProps) {
    let phase = p.state.boss.presentation;
    let on = p.on_hearing.clone();
    let paused = p.paused;
    use_effect_with((phase, p.fast, paused), move |(phase, fast, paused)| {
        let window = web_sys::window();
        let reduced = window
            .as_ref()
            .and_then(|w| {
                w.match_media("(prefers-reduced-motion: reduce)")
                    .ok()
                    .flatten()
            })
            .is_some_and(|m| m.matches());
        let phase = *phase;
        let callback = Closure::wrap(Box::new(move || {
            if crate::app::flow::visible() {
                on.emit(HearingCommand::Advance(phase));
            }
        }) as Box<dyn FnMut()>);
        let id = if *paused {
            None
        } else {
            duration(phase, *fast, reduced).and_then(|ms| {
                window.as_ref().and_then(|w| {
                    w.set_interval_with_callback_and_timeout_and_arguments_0(
                        callback.as_ref().unchecked_ref(),
                        ms,
                    )
                    .ok()
                })
            })
        };
        move || {
            if let (Some(w), Some(id)) = (window, id) {
                w.clear_interval_with_handle(id);
            }
            drop(callback);
        }
    });
    use_effect_with((phase, paused), move |(phase, paused)| {
        if !paused && let Some(window) = web_sys::window() {
            if matches!(
                phase,
                HearingPhase::Arrival
                    | HearingPhase::Preparation
                    | HearingPhase::RoundRolling(_)
                    | HearingPhase::VoteRolling
                    | HearingPhase::Verdict
            ) {
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
            if duration(*phase, false, false).is_none()
                && let Some(button) = window
                    .document()
                    .and_then(|d| d.get_element_by_id("hearing-next"))
                && let Ok(button) = button.dyn_into::<web_sys::HtmlElement>()
            {
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                let _ = button.focus_with_options(&options);
            }
        }
    });
}
