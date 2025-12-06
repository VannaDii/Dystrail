use crate::a11y::set_status;
use crate::game::{
    CampConfig, CampOutcome, GameState, camp_forage, camp_repair_hack, camp_repair_spare,
    camp_rest, camp_therapy, can_repair, can_therapy,
};
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::vehicle::{Breakdown, Part};
    use futures::executor::block_on;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    fn base_props(state: GameState) -> Props {
        Props {
            game_state: Rc::new(state),
            camp_config: Rc::new(CampConfig::default_config()),
            on_state_change: Callback::from(|_: GameState| {}),
            on_close: Callback::noop(),
        }
    }

    #[test]
    fn camp_panel_main_view_renders_actions() {
        crate::i18n::set_lang("en");
        let props = base_props(GameState::default());

        let html = block_on(LocalServerRenderer::<CampPanel>::with_props(props).render());
        assert!(
            html.contains("Rest") && html.contains("Forage"),
            "main view should list key camp actions: {html}"
        );
    }

    #[test]
    fn camp_panel_with_breakdown_starts_in_repair_view() {
        crate::i18n::set_lang("en");
        let props = base_props(GameState {
            travel_blocked: true,
            breakdown: Some(Breakdown {
                part: Part::Battery,
                day_started: 3,
            }),
            ..GameState::default()
        });

        let html = block_on(LocalServerRenderer::<CampPanel>::with_props(props).render());
        assert!(
            html.contains("Repair Vehicle") || html.contains("Use Spare"),
            "repair menu should surface when breakdown present: {html}"
        );
    }
}

#[derive(Clone, PartialEq)]
enum CampView {
    Main,
    Repair,
}

#[function_component(CampPanel)]
pub fn camp_panel(p: &Props) -> Html {
    let start_view = if p.game_state.breakdown.is_some() && p.game_state.travel_blocked {
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

    // Action handler
    let on_action = {
        let game_state = p.game_state.clone();
        let camp_config = p.camp_config.clone();
        let on_state_change = p.on_state_change.clone();
        let on_close = p.on_close.clone();
        let view_setter = current_view.setter();
        let view_current = (*current_view).clone();
        let status_setter = status_msg.setter();

        Callback::from(move |action: u8| {
            let mut new_state = (*game_state).clone();
            let outcome = match (&view_current, action) {
                (CampView::Main, 1) => {
                    // Rest
                    camp_rest(&mut new_state, &camp_config)
                }
                (CampView::Main, 2) => {
                    // Repair Vehicle
                    if can_repair(&new_state, &camp_config) {
                        view_setter.set(CampView::Repair);
                        return;
                    }
                    CampOutcome {
                        message: i18n::t("camp.announce.no_breakdown"),
                        rested: false,
                        supplies_delta: 0,
                    }
                }
                (CampView::Main, 3) => {
                    // Forage
                    camp_forage(&mut new_state, &camp_config)
                }
                (CampView::Main, 4) => {
                    // Therapy
                    camp_therapy(&mut new_state, &camp_config)
                }
                (CampView::Main, 0) => {
                    // Close
                    on_close.emit(());
                    return;
                }
                (CampView::Repair, 1) => {
                    // Use Spare
                    if let Some(breakdown) = &new_state.breakdown {
                        let part = breakdown.part;
                        let result = camp_repair_spare(&mut new_state, &camp_config, part);
                        view_setter.set(CampView::Main);
                        result
                    } else {
                        CampOutcome {
                            message: i18n::t("camp.announce.no_breakdown"),
                            rested: false,
                            supplies_delta: 0,
                        }
                    }
                }
                (CampView::Repair, 2) => {
                    // Hack Fix
                    let result = camp_repair_hack(&mut new_state, &camp_config);
                    view_setter.set(CampView::Main);
                    result
                }
                (CampView::Repair, 0) => {
                    // Back to main
                    view_setter.set(CampView::Main);
                    return;
                }
                _ => return,
            };

            status_setter.set(outcome.message.clone());
            set_status(&outcome.message);
            on_state_change.emit(new_state);

            // Close modal if action advanced the day
            if matches!(&view_current, CampView::Repair)
                || (matches!(action, 1 | 3 | 4) && matches!(&view_current, CampView::Main))
            {
                on_close.emit(());
            }
        })
    };

    // Keyboard handler
    let on_keydown = {
        let on_action = on_action.clone();
        let focus_idx = focus_idx.clone();
        let on_close = p.on_close.clone();
        let view_state = (*current_view).clone();

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
