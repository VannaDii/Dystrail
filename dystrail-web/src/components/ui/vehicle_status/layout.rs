use super::logic::SelectionResolution;
use super::menu_item::VehicleMenuItem;
use crate::a11y::set_status;
use crate::game::Part;
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
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
