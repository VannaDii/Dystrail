mod actions;
mod main_view;
mod repair_view;
#[cfg(test)]
mod tests;

use crate::game::{CampConfig, EndgameTravelCfg, GameState, can_repair, can_therapy};
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use actions::build_on_action;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct Props {
    pub game_state: Rc<GameState>,
    pub camp_config: Rc<CampConfig>,
    pub endgame_config: Rc<EndgameTravelCfg>,
    pub on_state_change: Callback<GameState>,
    pub on_close: Callback<()>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.game_state, &other.game_state)
            && Rc::ptr_eq(&self.camp_config, &other.camp_config)
            && Rc::ptr_eq(&self.endgame_config, &other.endgame_config)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CampView {
    Main,
    Repair,
}

#[function_component(CampPanel)]
pub fn camp_panel(p: &Props) -> Html {
    let start_view =
        if p.game_state.breakdown.is_some() && p.game_state.day_state.travel.travel_blocked {
            CampView::Repair
        } else {
            CampView::Main
        };
    let current_view = use_state(move || start_view);
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();
    let status_msg = use_state(String::new);

    {
        let list_ref = list_ref.clone();
        use_effect_with(*focus_idx, move |idx| {
            if let Some(list) = list_ref.cast::<web_sys::Element>() {
                let sel = format!("[role='menuitem'][data-key='{idx}']");
                if let Ok(Some(el)) = list.query_selector(&sel) {
                    let _ = el
                        .dyn_into::<web_sys::HtmlElement>()
                        .ok()
                        .map(|e| e.focus());
                }
            }
        });
    }

    let on_action = build_on_action(
        p.game_state.clone(),
        p.camp_config.clone(),
        p.endgame_config.clone(),
        p.on_state_change.clone(),
        p.on_close.clone(),
        &current_view,
        &status_msg,
    );

    let on_keydown = {
        let on_action = on_action.clone();
        let focus_idx = focus_idx.clone();
        let on_close = p.on_close.clone();
        let view_state = *current_view;

        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();

            if let Some(n) = numeric_key_to_index(&key) {
                on_action.emit(n);
                e.prevent_default();
                return;
            }

            if let Some(n) = numeric_code_to_index(&e.code()) {
                on_action.emit(n);
                e.prevent_default();
                return;
            }

            match key.as_str() {
                "Enter" | " " => {
                    on_action.emit(*focus_idx);
                    e.prevent_default();
                }
                "Escape" => {
                    on_close.emit(());
                    e.prevent_default();
                }
                "ArrowDown" => {
                    let max = match view_state {
                        CampView::Main => 4,
                        CampView::Repair => 2,
                    };
                    let mut next = *focus_idx + 1;
                    if next > max {
                        next = 0;
                    }
                    focus_idx.set(next);
                    e.prevent_default();
                }
                "ArrowUp" => {
                    let max = match view_state {
                        CampView::Main => 4,
                        CampView::Repair => 2,
                    };
                    let mut prev = if *focus_idx == 0 { max } else { *focus_idx - 1 };
                    if prev == 0 {
                        prev = max;
                    }
                    focus_idx.set(prev);
                    e.prevent_default();
                }
                _ => {}
            }
        })
    };

    match &*current_view {
        CampView::Main => {
            let can_repair_now = can_repair(&p.game_state, &p.camp_config);
            let can_therapy_now = can_therapy(&p.game_state, &p.camp_config);
            let status_text = (*status_msg).clone();
            main_view::render_main(
                *focus_idx,
                &list_ref,
                &on_action,
                &on_keydown,
                &status_text,
                can_repair_now,
                can_therapy_now,
            )
        }
        CampView::Repair => {
            let breakdown = p.game_state.breakdown.as_ref();
            let status_text = (*status_msg).clone();
            repair_view::render_repair(
                *focus_idx,
                &list_ref,
                &on_action,
                &on_keydown,
                &status_text,
                breakdown,
            )
        }
    }
}
