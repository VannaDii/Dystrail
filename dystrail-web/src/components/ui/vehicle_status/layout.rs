use super::logic::SelectionResolution;
use super::menu_item::VehicleMenuItem;
use crate::a11y::set_status;
use crate::game::Part;
use crate::i18n;
#[cfg(target_arch = "wasm32")]
use crate::input::{numeric_code_to_index, numeric_key_to_index};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

pub fn menu_items(
    breakdown_part: Option<Part>,
    spare_counts: Option<(i32, i32, i32, i32)>,
) -> Vec<(u8, (String, bool))> {
    vec![
        (1_u8, {
            let count = spare_counts.map_or(0, |(tire, _, _, _)| tire);
            let enabled = breakdown_part == Some(Part::Tire) && count > 0;
            (
                format!(
                    "{tire} (x{count})",
                    tire = i18n::t("vehicle.spares.tire"),
                    count = count
                ),
                enabled,
            )
        }),
        (2, {
            let count = spare_counts.map_or(0, |(_, battery, _, _)| battery);
            let enabled = breakdown_part == Some(Part::Battery) && count > 0;
            (
                format!(
                    "{battery} (x{count})",
                    battery = i18n::t("vehicle.spares.battery"),
                    count = count
                ),
                enabled,
            )
        }),
        (3, {
            let count = spare_counts.map_or(0, |(_, _, alt, _)| alt);
            let enabled = breakdown_part == Some(Part::Alternator) && count > 0;
            (
                format!(
                    "{alt} (x{count})",
                    alt = i18n::t("vehicle.spares.alt"),
                    count = count
                ),
                enabled,
            )
        }),
        (4, {
            let count = spare_counts.map_or(0, |(_, _, _, pump)| pump);
            let enabled = breakdown_part == Some(Part::FuelPump) && count > 0;
            (
                format!(
                    "{pump} (x{count})",
                    pump = i18n::t("vehicle.spares.pump"),
                    count = count
                ),
                enabled,
            )
        }),
        (5, {
            let enabled = breakdown_part.is_some();
            (i18n::t("vehicle.hack_fix"), enabled)
        }),
        (0, (i18n::t("vehicle.back"), true)),
    ]
}

pub fn status_message(status: Option<Part>) -> String {
    status.map_or_else(
        || i18n::t("vehicle.no_active"),
        |part| {
            let mut vars = std::collections::BTreeMap::new();
            let part_name = i18n::t(part.key());
            vars.insert("part", part_name.as_str());
            i18n::tr("vehicle.breakdown", Some(&vars))
        },
    )
}

#[cfg(target_arch = "wasm32")]
pub fn focus_effect(list_ref: NodeRef, focus_idx: &UseStateHandle<u8>) {
    let focus_idx = focus_idx.clone();
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

#[cfg(not(target_arch = "wasm32"))]
pub fn focus_effect(list_ref: NodeRef, focus_idx: &UseStateHandle<u8>) {
    let _ = (list_ref, focus_idx);
}

#[cfg(target_arch = "wasm32")]
pub fn keydown_handler(
    activate: Callback<u8>,
    focus_idx: UseStateHandle<u8>,
) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
        let key = e.key();
        if let Some(n) = numeric_key_to_index(&key) {
            activate.emit(n);
            e.prevent_default();
            return;
        }
        if let Some(n) = numeric_code_to_index(&e.code()) {
            activate.emit(n);
            e.prevent_default();
            return;
        }
        if key == "Enter" || key == " " {
            activate.emit(*focus_idx);
            e.prevent_default();
        } else if key == "Escape" {
            activate.emit(0);
            e.prevent_default();
        } else if key == "ArrowDown" {
            let mut next = *focus_idx + 1;
            if next > 5 {
                next = 0;
            }
            focus_idx.set(next);
            e.prevent_default();
        } else if key == "ArrowUp" {
            let mut prev = if *focus_idx == 0 { 5 } else { *focus_idx - 1 };
            if prev == 0 {
                prev = 5;
            }
            focus_idx.set(prev);
            e.prevent_default();
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn keydown_handler(
    activate: Callback<u8>,
    focus_idx: UseStateHandle<u8>,
) -> Callback<KeyboardEvent> {
    let _ = (activate, focus_idx);
    Callback::from(|_e: KeyboardEvent| {})
}

pub fn render_menu(
    items: &[(u8, (String, bool))],
    focus_idx: u8,
    setsize: u8,
    activate: &Callback<u8>,
) -> Html {
    items
        .iter()
        .enumerate()
        .map(|(i, (idx, (label, enabled)))| {
            let focused = focus_idx == *idx;
            let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);
            html! { <VehicleMenuItem
                index={*idx}
                posinset={posinset}
                label={AttrValue::from(label.clone())}
                focused={focused}
                disabled={!enabled}
                setsize={setsize}
                on_activate={activate.clone()}
            /> }
        })
        .collect()
}

pub fn resolve_selection(
    on_back: Callback<()>,
    on_repair: Callback<super::logic::VehicleAction>,
    breakdown_part: Option<Part>,
    spare_counts: Option<(i32, i32, i32, i32)>,
) -> Callback<u8> {
    Callback::from(move |idx: u8| {
        match super::logic::evaluate_selection(idx, breakdown_part, spare_counts) {
            SelectionResolution::Action(action, message) => {
                on_repair.emit(action);
                set_status(&message);
            }
            SelectionResolution::Message(message) => set_status(&message),
            SelectionResolution::Back => {
                on_back.emit(());
                set_status(&i18n::t("menu.back"));
            }
            SelectionResolution::None => {}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ui::vehicle_status::logic::VehicleAction;
    use futures::executor::block_on;
    use std::cell::RefCell;
    use std::rc::Rc;
    use yew::LocalServerRenderer;

    #[test]
    fn menu_items_enable_correct_entries() {
        crate::i18n::set_lang("en");
        let items = menu_items(Some(Part::Tire), Some((1, 0, 0, 0)));
        let tire = items.iter().find(|(idx, _)| *idx == 1).unwrap();
        assert!(tire.1.1);
        let battery = items.iter().find(|(idx, _)| *idx == 2).unwrap();
        assert!(!battery.1.1);
    }

    #[test]
    fn status_message_defaults_when_none() {
        crate::i18n::set_lang("en");
        let msg = status_message(None);
        assert!(!msg.is_empty());
    }

    #[test]
    fn status_message_includes_part() {
        crate::i18n::set_lang("en");
        let msg = status_message(Some(Part::FuelPump));
        let part_name = i18n::t(Part::FuelPump.key());
        assert!(msg.contains(&part_name));
    }

    #[test]
    fn menu_items_cover_all_spares() {
        crate::i18n::set_lang("en");
        let items = menu_items(Some(Part::Battery), Some((0, 2, 0, 0)));
        let battery = items.iter().find(|(idx, _)| *idx == 2).unwrap();
        assert!(battery.1.1);
        let tire = items.iter().find(|(idx, _)| *idx == 1).unwrap();
        assert!(!tire.1.1);
    }

    #[test]
    fn menu_items_disable_actions_without_breakdown() {
        crate::i18n::set_lang("en");
        let items = menu_items(None, Some((1, 1, 1, 1)));
        let hack = items.iter().find(|(idx, _)| *idx == 5).unwrap();
        assert!(!hack.1.1);
        let back = items.iter().find(|(idx, _)| *idx == 0).unwrap();
        assert!(back.1.1);
    }

    #[function_component(TestMenu)]
    fn test_menu() -> Html {
        let items = menu_items(Some(Part::Tire), Some((1, 0, 0, 0)));
        let activate = Callback::from(|_| ());
        html! { <ul>{ render_menu(&items, 1, 6, &activate) }</ul> }
    }

    #[function_component(TestFocusKeydown)]
    fn test_focus_keydown() -> Html {
        let focus_idx = use_state(|| 1_u8);
        let list_ref = use_node_ref();
        focus_effect(list_ref.clone(), &focus_idx);
        let _handler = keydown_handler(Callback::from(|_| ()), focus_idx);
        html! { <div ref={list_ref}></div> }
    }

    #[test]
    fn render_menu_outputs_items() {
        crate::i18n::set_lang("en");
        let html = block_on(LocalServerRenderer::<TestMenu>::new().render());
        assert!(html.contains("ot-menuitem"));
    }

    #[test]
    fn focus_and_keydown_handlers_render() {
        let html = block_on(LocalServerRenderer::<TestFocusKeydown>::new().render());
        assert!(html.contains("<div"));
    }

    #[test]
    fn resolve_selection_emits_callbacks() {
        crate::i18n::set_lang("en");
        let action_slot: Rc<RefCell<Option<VehicleAction>>> = Rc::new(RefCell::new(None));
        let action_slot_clone = action_slot.clone();
        let back_hit = Rc::new(RefCell::new(false));
        let back_hit_clone = back_hit.clone();
        let on_back = Callback::from(move |()| {
            *back_hit_clone.borrow_mut() = true;
        });
        let on_repair = Callback::from(move |action| {
            *action_slot_clone.borrow_mut() = Some(action);
        });

        let callback = resolve_selection(on_back, on_repair, Some(Part::Tire), Some((1, 0, 0, 0)));
        callback.emit(1);
        assert!(matches!(
            *action_slot.borrow(),
            Some(VehicleAction::UseSpare(Part::Tire))
        ));

        callback.emit(0);
        assert!(*back_hit.borrow());
    }

    #[test]
    fn resolve_selection_emits_message_only() {
        crate::i18n::set_lang("en");
        let action_slot: Rc<RefCell<Option<VehicleAction>>> = Rc::new(RefCell::new(None));
        let action_slot_clone = action_slot.clone();
        let back_hit = Rc::new(RefCell::new(false));
        let back_hit_clone = back_hit.clone();
        let on_back = Callback::from(move |()| {
            *back_hit_clone.borrow_mut() = true;
        });
        let on_repair = Callback::from(move |action| {
            *action_slot_clone.borrow_mut() = Some(action);
        });

        let callback = resolve_selection(on_back, on_repair, Some(Part::Tire), Some((0, 0, 0, 0)));
        callback.emit(1);
        assert!(action_slot.borrow().is_none());
        assert!(!*back_hit.borrow());
    }

    #[test]
    fn resolve_selection_ignores_unknown_index() {
        crate::i18n::set_lang("en");
        let action_slot: Rc<RefCell<Option<VehicleAction>>> = Rc::new(RefCell::new(None));
        let action_slot_clone = action_slot.clone();
        let back_hit = Rc::new(RefCell::new(false));
        let back_hit_clone = back_hit.clone();
        let on_back = Callback::from(move |()| {
            *back_hit_clone.borrow_mut() = true;
        });
        let on_repair = Callback::from(move |action| {
            *action_slot_clone.borrow_mut() = Some(action);
        });

        let callback = resolve_selection(on_back, on_repair, None, None);
        callback.emit(9);
        assert!(action_slot.borrow().is_none());
        assert!(!*back_hit.borrow());
    }
}
