mod line;
mod selection;
#[cfg(test)]
mod tests;

use crate::game::{DietId, GameState, PaceId, PacingConfig};
use crate::i18n;
use crate::input::numeric_key_to_index;
use line::MenuLine;
use std::rc::Rc;
use yew::prelude::*;

pub use selection::{SelectionOutcome, selection_outcome};

#[derive(Properties)]
pub struct PaceDietPanelProps {
    pub game_state: Rc<GameState>,
    pub pacing_config: Rc<PacingConfig>,
    pub on_pace_change: Callback<PaceId>,
    pub on_diet_change: Callback<DietId>,
    pub on_back: Callback<()>,
}

impl PartialEq for PaceDietPanelProps {
    fn eq(&self, other: &Self) -> bool {
        self.game_state.pace == other.game_state.pace
            && self.game_state.diet == other.game_state.diet
    }
}

#[function_component(PaceDietPanel)]
pub fn pace_diet_panel(props: &PaceDietPanelProps) -> Html {
    let focused_index = use_state(|| 1u8);
    let status_message = use_state(String::new);

    let on_activate = {
        let pacing_config = props.pacing_config.clone();
        let on_pace_change = props.on_pace_change.clone();
        let on_diet_change = props.on_diet_change.clone();
        let on_back = props.on_back.clone();
        let status_message = status_message.clone();

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
    };

    let on_keydown = {
        let focused_index = focused_index.clone();
        let on_activate = on_activate.clone();

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
    };

    let on_focus = {
        let focused_index = focused_index.clone();
        Callback::from(move |idx: u8| focused_index.set(idx))
    };

    let current_pace = props.game_state.pace;
    let current_diet = props.game_state.diet;

    let options = vec![
        (
            1,
            i18n::t("pacediet.menu.pace_steady"),
            current_pace == PaceId::Steady,
            i18n::t("pacediet.tooltips.steady"),
        ),
        (
            2,
            i18n::t("pacediet.menu.pace_heated"),
            current_pace == PaceId::Heated,
            i18n::t("pacediet.tooltips.heated"),
        ),
        (
            3,
            i18n::t("pacediet.menu.pace_blitz"),
            current_pace == PaceId::Blitz,
            i18n::t("pacediet.tooltips.blitz"),
        ),
        (
            4,
            i18n::t("pacediet.menu.diet_quiet"),
            current_diet == DietId::Quiet,
            i18n::t("pacediet.tooltips.quiet"),
        ),
        (
            5,
            i18n::t("pacediet.menu.diet_mixed"),
            current_diet == DietId::Mixed,
            i18n::t("pacediet.tooltips.mixed"),
        ),
        (
            6,
            i18n::t("pacediet.menu.diet_doom"),
            current_diet == DietId::Doom,
            i18n::t("pacediet.tooltips.doom"),
        ),
        (
            0,
            i18n::t("pacediet.menu.back"),
            false,
            i18n::t("pacediet.menu.back"),
        ),
    ];

    html! {
        <section
            role="region"
            aria-labelledby="pd-title"
            onkeydown={on_keydown}
            class="pace-diet-panel"
        >
            <h3 id="pd-title" class="pace-diet-title">
                {i18n::t("pacediet.title")}
            </h3>

            <ul
                role="menu"
                aria-label={i18n::t("pacediet.title")}
                class="pace-diet-menu"
            >
                { for options.into_iter().map(|(idx, text, selected, tooltip)| html! {
                    <MenuLine
                        key={idx}
                        index={idx}
                        text={text.clone()}
                        selected={selected}
                        focused={*focused_index == idx}
                        on_activate={on_activate.clone()}
                        on_focus={on_focus.clone()}
                        tooltip={tooltip}
                    />
                }) }
            </ul>

            <div
                id="pd-status"
                aria-live="polite"
                class="pace-diet-status"
                role="status"
            >
                {(*status_message).clone()}
            </div>
        </section>
    }
}
