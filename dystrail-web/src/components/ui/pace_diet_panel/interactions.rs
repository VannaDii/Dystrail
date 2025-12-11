use super::selection::{SelectionOutcome, selection_outcome};
use crate::game::{DietId, PaceId, PacingConfig};
use crate::input::numeric_key_to_index;
use std::rc::Rc;
use yew::prelude::*;

pub fn activate_handler(
    pacing_config: Rc<PacingConfig>,
    on_pace_change: Callback<PaceId>,
    on_diet_change: Callback<DietId>,
    on_back: Callback<()>,
    status_message: UseStateHandle<String>,
) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        if idx == 0 {
            status_message.set(String::new());
            on_back.emit(());
            return;
        }

        if let Some(outcome) = selection_outcome(&pacing_config, idx) {
            match outcome {
                SelectionOutcome::Pace(pace, announcement) => {
                    status_message.set(announcement);
                    on_pace_change.emit(pace);
                }
                SelectionOutcome::Diet(diet, announcement) => {
                    status_message.set(announcement);
                    on_diet_change.emit(diet);
                }
            }
        }
    })
}

pub fn focus_handler(focused_index: UseStateHandle<u8>) -> Callback<u8> {
    Callback::from(move |idx: u8| focused_index.set(idx))
}

pub fn keydown_handler(
    focused_index: UseStateHandle<u8>,
    on_activate: Callback<u8>,
) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| match e.key().as_str() {
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
            if let Some(n) = numeric_key_to_index(e.key().as_str()) {
                on_activate.emit(n);
                e.prevent_default();
            }
        }
        "ArrowDown" => {
            let current = *focused_index;
            let next = if current >= 6 { 0 } else { current + 1 };
            focused_index.set(next);
            e.prevent_default();
        }
        "ArrowUp" => {
            let current = *focused_index;
            let next = if current == 0 { 6 } else { current - 1 };
            focused_index.set(next);
            e.prevent_default();
        }
        "Enter" | " " => {
            on_activate.emit(*focused_index);
            e.prevent_default();
        }
        "Escape" => {
            on_activate.emit(0);
            e.prevent_default();
        }
        _ => {}
    })
}
