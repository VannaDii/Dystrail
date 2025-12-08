use crate::game::{CampConfig, GameState, can_repair, can_therapy};
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use actions::build_on_action;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

mod actions;
#[derive(Properties, Clone)]
pub struct Props {
    pub game_state: Rc<GameState>,
    pub camp_config: Rc<CampConfig>,
    pub on_state_change: Callback<GameState>,
    pub on_close: Callback<()>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.game_state, &other.game_state)
            && Rc::ptr_eq(&self.camp_config, &other.camp_config)
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

    // Set up focus management when focus index changes
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
        p.on_state_change.clone(),
        p.on_close.clone(),
        &current_view,
        &status_msg,
    );

    // Keyboard handler
    let on_keydown = {
        let on_action = on_action.clone();
        let focus_idx = focus_idx.clone();
        let on_close = p.on_close.clone();
        let view_state = *current_view;

        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();

            // Direct numeric activation
            if let Some(n) = numeric_key_to_index(&key) {
                on_action.emit(n);
                e.prevent_default();
                return;
            }

            // Use code (DigitN/NumpadN) as fallback
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

            let items = vec![
                (1_u8, i18n::t("camp.menu.rest"), true),
                (2, i18n::t("camp.menu.repair"), true),
                (3, i18n::t("camp.menu.forage"), true),
                (4, i18n::t("camp.menu.therapy"), can_therapy_now),
                (0, i18n::t("camp.menu.close"), true),
            ];

            html! {
                <section
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="camp-title"
                    aria-describedby="camp-desc"
                    onkeydown={on_keydown}
                    class="ot-menu camp-modal"
                    tabindex="0"
                >
                    <h2 id="camp-title">{ i18n::t("camp.title") }</h2>
                    <p id="camp-desc" class="sr-only">{ i18n::t("camp.desc") }</p>

                    <ul role="menu" aria-label={i18n::t("camp.title")} ref={list_ref}>
                        { for items.into_iter().enumerate().map(|(i, (idx, label, enabled))| {
                            let focused = *focus_idx == idx;
                            let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);
                            let disabled_class = if enabled { "" } else { "disabled" };
                            let disabled_attr = if enabled { "false" } else { "true" };

                            html! {
                                <li
                                    role="menuitem"
                                    tabindex={ if focused { "0" } else { "-1" } }
                                    data-key={idx.to_string()}
                                    aria-posinset={posinset.to_string()}
                                    aria-setsize="5"
                                    aria-disabled={disabled_attr}
                                    onclick={
                                        let on_action = on_action.clone();
                                        Callback::from(move |_| on_action.emit(idx))
                                    }
                                    class={format!("ot-menuitem {disabled_class}")}
                                >
                                    <span class="num">{ format!("{idx})") }</span>
                                    <span class="label">{ label }</span>
                                    { if idx == 2 && !can_repair_now {
                                        html!{ <span class="note">{ format!(" ({note})", note = i18n::t("camp.announce.no_breakdown")) }</span> }
                                    } else { html!{} } }
                                </li>
                            }
                        }) }
                    </ul>

                    <p aria-live="polite" class="status">{ (*status_msg).clone() }</p>
                </section>
            }
        }
        CampView::Repair => {
            let breakdown = p.game_state.breakdown.as_ref();
            let part_name =
                breakdown.map_or_else(|| "Unknown".to_string(), |b| i18n::t(b.part.key()));

            let items = vec![
                (1_u8, i18n::t("camp.menu.use_spare"), true),
                (2, i18n::t("camp.menu.hack_fix"), true),
                (0, i18n::t("camp.menu.back"), true),
            ];

            html! {
                <section
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="repair-title"
                    onkeydown={on_keydown}
                    class="ot-menu repair-modal"
                    tabindex="0"
                >
                    <h2 id="repair-title">{ i18n::t("camp.repair.title") }</h2>
                    <p>{ format!("{breakdown_label}: {part_name}", breakdown_label = i18n::t("vehicle.breakdown")) }</p>

                    <ul role="menu" aria-label={i18n::t("camp.repair.title")} ref={list_ref}>
                        { for items.into_iter().enumerate().map(|(i, (idx, label, _enabled))| {
                            let focused = *focus_idx == idx;
                            let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);

                            html! {
                                <li
                                    role="menuitem"
                                    tabindex={ if focused { "0" } else { "-1" } }
                                    data-key={idx.to_string()}
                                    aria-posinset={posinset.to_string()}
                                    aria-setsize="3"
                                    onclick={
                                        let on_action = on_action.clone();
                                        Callback::from(move |_| on_action.emit(idx))
                                    }
                                    class="ot-menuitem"
                                >
                                    <span class="num">{ format!("{idx})") }</span>
                                    <span class="label">{ label }</span>
                                </li>
                            }
                        }) }
                    </ul>

                    <p aria-live="polite" class="status">{ (*status_msg).clone() }</p>
                </section>
            }
        }
    }
}

#[cfg(test)]
mod tests;
