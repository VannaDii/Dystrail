mod interactions;
mod line;
mod options;
mod selection;
#[cfg(test)]
mod tests;

use crate::game::{DietId, GameState, PaceId, PacingConfig};
use crate::i18n;
use interactions::{activate_handler, focus_handler, keydown_handler};
use line::MenuLine;
use options::menu_options;
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
    let focused_index = use_state(|| 1_u8);
    let status_message = use_state(String::new);

    let on_activate = activate_handler(
        props.pacing_config.clone(),
        props.on_pace_change.clone(),
        props.on_diet_change.clone(),
        props.on_back.clone(),
        status_message.clone(),
    );

    let on_keydown = keydown_handler(focused_index.clone(), on_activate.clone());
    let on_focus = focus_handler(focused_index.clone());

    let current_pace = props.game_state.pace;
    let current_diet = props.game_state.diet;
    let options = menu_options(current_pace, current_diet);

    html! {
        <section
            role="region"
            aria-labelledby="pd-title"
            onkeydown={on_keydown}
            class="pace-diet-panel"
        >
            <h3 id="pd-title" class="pace-diet-title">
                { i18n::t("pacediet.title") }
            </h3>

            <ul
                role="menu"
                aria-label={i18n::t("pacediet.title")}
                class="pace-diet-menu"
            >
                { for options.into_iter().map(|opt| html! {
                    <MenuLine
                        key={opt.idx}
                        index={opt.idx}
                        text={opt.text.clone()}
                        selected={opt.selected}
                        focused={*focused_index == opt.idx}
                        on_activate={on_activate.clone()}
                        on_focus={on_focus.clone()}
                        tooltip={opt.tooltip}
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
