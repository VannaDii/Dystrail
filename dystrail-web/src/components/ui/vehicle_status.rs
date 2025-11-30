use crate::a11y::set_status;
use crate::game::{GameState, Part};
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct VehicleMenuItemProps {
    pub index: u8,        // 0..9
    pub label: AttrValue, // resolved string
    pub focused: bool,    // tabindex 0 vs -1
    pub disabled: bool,   // aria-disabled
    pub posinset: u8,     // 1..=setsize
    pub setsize: u8,
    pub on_activate: Callback<u8>, // called with index
}

#[function_component(VehicleMenuItem)]
pub fn vehicle_menu_item(p: &VehicleMenuItemProps) -> Html {
    let idx = p.index;
    let on_click = {
        let on = p.on_activate.clone();
        let disabled = p.disabled;
        Callback::from(move |_| {
            if !disabled {
                on.emit(idx);
            }
        })
    };

    let classes = if p.disabled {
        "ot-menuitem disabled"
    } else {
        "ot-menuitem"
    };

    html! {
      <li role="menuitem"
          tabindex={ if p.focused { "0" } else { "-1" } }
          data-key={idx.to_string()}
          aria-posinset={p.posinset.to_string()}
          aria-setsize={p.setsize.to_string()}
          aria-disabled={p.disabled.to_string()}
          onclick={on_click}
          class={classes}>
         <span class="num">{ format!("{idx})") }</span>
         <span class="label">{ p.label.clone() }</span>
      </li>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_selection_uses_spare_when_available() {
        crate::i18n::set_lang("en");
        let outcome = evaluate_selection(1, Some(Part::Tire), Some((1, 0, 0, 0)));
        assert!(matches!(
            outcome,
            SelectionResolution::Action(VehicleAction::UseSpare(Part::Tire), _)
        ));
    }

    #[test]
    fn evaluate_selection_reports_missing_spare() {
        crate::i18n::set_lang("en");
        let outcome = evaluate_selection(2, Some(Part::Battery), Some((0, 0, 0, 0)));
        assert!(matches!(outcome, SelectionResolution::Message(_)));
    }

    #[test]
    fn evaluate_selection_handles_hack_fix() {
        crate::i18n::set_lang("en");
        let with_breakdown = evaluate_selection(5, Some(Part::FuelPump), None);
        assert!(matches!(
            with_breakdown,
            SelectionResolution::Action(VehicleAction::HackFix, _)
        ));

        let without = evaluate_selection(5, None, None);
        assert!(matches!(without, SelectionResolution::Message(_)));
    }

    #[test]
    fn evaluate_selection_back_option() {
        crate::i18n::set_lang("en");
        assert_eq!(evaluate_selection(0, None, None), SelectionResolution::Back);
    }
}

#[derive(Properties, Clone)]
pub struct VehicleStatusProps {
    pub game_state: Option<Rc<GameState>>,
    pub on_back: Callback<()>,
    pub on_repair_action: Callback<VehicleAction>,
}

impl PartialEq for VehicleStatusProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare by game state fields that matter for re-rendering
        let self_breakdown = self
            .game_state
            .as_ref()
            .and_then(|gs| gs.breakdown.as_ref().map(|b| (b.part, b.day_started)));
        let other_breakdown = other
            .game_state
            .as_ref()
            .and_then(|gs| gs.breakdown.as_ref().map(|b| (b.part, b.day_started)));

        let self_spares = self.game_state.as_ref().map(|gs| {
            (
                gs.inventory.spares.tire,
                gs.inventory.spares.battery,
                gs.inventory.spares.alt,
                gs.inventory.spares.pump,
            )
        });
        let other_spares = other.game_state.as_ref().map(|gs| {
            (
                gs.inventory.spares.tire,
                gs.inventory.spares.battery,
                gs.inventory.spares.alt,
                gs.inventory.spares.pump,
            )
        });

        self_breakdown == other_breakdown && self_spares == other_spares
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleAction {
    UseSpare(Part),
    HackFix,
}

#[derive(Debug, PartialEq)]
enum SelectionResolution {
    Action(VehicleAction, String),
    Message(String),
    Back,
    None,
}

fn evaluate_selection(
    idx: u8,
    breakdown_part: Option<Part>,
    spare_counts: Option<(i32, i32, i32, i32)>,
) -> SelectionResolution {
    let used_spare_message = |part: Part| {
        let part_name = i18n::t(part.key());
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("part", part_name.as_str());
        vars.insert("sup", "1");
        i18n::tr("vehicle.announce.used_spare", Some(&vars))
    };
    let missing_spare_message = |part: Part| {
        let part_name = i18n::t(part.key());
        let mut vars = std::collections::BTreeMap::new();
        vars.insert("part", part_name.as_str());
        i18n::tr("vehicle.announce.no_spare", Some(&vars))
    };

    match idx {
        1 => match (breakdown_part, spare_counts) {
            (Some(Part::Tire), Some((tire, _, _, _))) if tire > 0 => SelectionResolution::Action(
                VehicleAction::UseSpare(Part::Tire),
                used_spare_message(Part::Tire),
            ),
            _ => SelectionResolution::Message(missing_spare_message(Part::Tire)),
        },
        2 => match (breakdown_part, spare_counts) {
            (Some(Part::Battery), Some((_, battery, _, _))) if battery > 0 => {
                SelectionResolution::Action(
                    VehicleAction::UseSpare(Part::Battery),
                    used_spare_message(Part::Battery),
                )
            }
            _ => SelectionResolution::Message(missing_spare_message(Part::Battery)),
        },
        3 => match (breakdown_part, spare_counts) {
            (Some(Part::Alternator), Some((_, _, alt, _))) if alt > 0 => {
                SelectionResolution::Action(
                    VehicleAction::UseSpare(Part::Alternator),
                    used_spare_message(Part::Alternator),
                )
            }
            _ => SelectionResolution::Message(missing_spare_message(Part::Alternator)),
        },
        4 => match (breakdown_part, spare_counts) {
            (Some(Part::FuelPump), Some((_, _, _, pump))) if pump > 0 => {
                SelectionResolution::Action(
                    VehicleAction::UseSpare(Part::FuelPump),
                    used_spare_message(Part::FuelPump),
                )
            }
            _ => SelectionResolution::Message(missing_spare_message(Part::FuelPump)),
        },
        5 => {
            if breakdown_part.is_some() {
                let mut vars = std::collections::BTreeMap::new();
                vars.insert("sup", "3");
                vars.insert("cred", "1");
                vars.insert("day", "1");
                SelectionResolution::Action(
                    VehicleAction::HackFix,
                    i18n::tr("vehicle.announce.hack_applied", Some(&vars)),
                )
            } else {
                SelectionResolution::Message(i18n::t("vehicle.no_active"))
            }
        }
        0 => SelectionResolution::Back,
        _ => SelectionResolution::None,
    }
}

#[function_component(VehicleStatus)]
pub fn vehicle_status(p: &VehicleStatusProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();

    let gs = p.game_state.as_ref();
    let breakdown = gs.and_then(|state| state.breakdown.as_ref());
    let spares = gs.map(|state| &state.inventory.spares);

    let activate = {
        let on_repair = p.on_repair_action.clone();
        let on_back = p.on_back.clone();
        let breakdown_part = breakdown.map(|b| b.part);
        let spare_counts = spares.map(|s| (s.tire, s.battery, s.alt, s.pump));

        Callback::from(
            move |idx: u8| match evaluate_selection(idx, breakdown_part, spare_counts) {
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
            },
        )
    };

    // When focus index changes, move DOM focus to the corresponding item
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

    let on_keydown = {
        let activate = activate.clone();
        let focus_idx = focus_idx.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            // Direct numeric activation
            if let Some(n) = numeric_key_to_index(&key) {
                activate.emit(n);
                e.prevent_default();
                return;
            }
            // Use code (DigitN/NumpadN) as fallback
            if let Some(n) = numeric_code_to_index(&e.code()) {
                activate.emit(n);
                e.prevent_default();
                return;
            }
            if key == "Enter" || key == " " {
                activate.emit(*focus_idx);
                e.prevent_default();
            } else if key == "Escape" {
                activate.emit(0); // Back
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
    };

    // Build menu items with proper disabled states
    let items = {
        let breakdown_part = breakdown.map(|b| b.part);
        let spare_counts = spares.map(|s| (s.tire, s.battery, s.alt, s.pump));

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
                let enabled = breakdown.is_some();
                (i18n::t("vehicle.hack_fix"), enabled)
            }),
            (0, (i18n::t("vehicle.back"), true)),
        ]
    };

    let setsize = u8::try_from(items.len()).unwrap_or(255);

    // Status message
    let status_msg = breakdown.map_or_else(
        || i18n::t("vehicle.no_active"),
        |breakdown| {
            let part_name = i18n::t(breakdown.part.key());
            let mut vars = std::collections::BTreeMap::new();
            vars.insert("part", part_name.as_str());
            i18n::tr("vehicle.breakdown", Some(&vars))
        },
    );

    html! {
      <section role="region" aria-labelledby="vehicle-title" onkeydown={on_keydown} class="ot-menu">
        <h2 id="vehicle-title">{ i18n::t("vehicle.title") }</h2>

        <div class="status-display" role="status" aria-live="polite">
            <p>{ status_msg }</p>
        </div>

        <ul role="menu" aria-label={i18n::t("vehicle.title")} id="vehicle-menu" ref={list_ref}>
          { for items.iter().enumerate().map(|(i, (idx, (label, enabled)))| {
              let focused = *focus_idx == *idx;
              let posinset = u8::try_from(i).unwrap_or(0) + 1;
              html!{ <VehicleMenuItem
                index={*idx}
                posinset={posinset}
                label={AttrValue::from(label.clone())}
                focused={focused}
                disabled={!enabled}
                setsize={setsize}
                on_activate={activate.clone()}
              /> }
          }) }
        </ul>

        <p class="muted">{ i18n::t("vehicle.help") }</p>
      </section>
    }
}
