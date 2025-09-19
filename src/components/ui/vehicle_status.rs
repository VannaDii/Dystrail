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
         <span class="num">{ format!("{}{})", idx, ")") }</span>
         <span class="label">{ p.label.clone() }</span>
      </li>
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

#[derive(Clone, PartialEq)]
pub enum VehicleAction {
    UseSpare(Part),
    HackFix,
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

        Callback::from(move |idx: u8| {
            match idx {
                1 => {
                    // Use Spare Tire
                    if breakdown_part == Some(Part::Tire) {
                        if let Some((tire_count, _, _, _)) = spare_counts {
                            if tire_count > 0 {
                                on_repair.emit(VehicleAction::UseSpare(Part::Tire));
                                let part_name = i18n::t(Part::Tire.key());
                                let mut vars = std::collections::HashMap::new();
                                vars.insert("part", part_name.as_str());
                                vars.insert("sup", "1");
                                set_status(&i18n::tr("vehicle.announce.used_spare", Some(&vars)));
                                return;
                            }
                        }
                    }
                    let part_name = i18n::t(Part::Tire.key());
                    let mut vars = std::collections::HashMap::new();
                    vars.insert("part", part_name.as_str());
                    set_status(&i18n::tr("vehicle.announce.no_spare", Some(&vars)));
                }
                2 => {
                    // Use Spare Battery
                    if breakdown_part == Some(Part::Battery) {
                        if let Some((_, battery_count, _, _)) = spare_counts {
                            if battery_count > 0 {
                                on_repair.emit(VehicleAction::UseSpare(Part::Battery));
                                let part_name = i18n::t(Part::Battery.key());
                                let mut vars = std::collections::HashMap::new();
                                vars.insert("part", part_name.as_str());
                                vars.insert("sup", "1");
                                set_status(&i18n::tr("vehicle.announce.used_spare", Some(&vars)));
                                return;
                            }
                        }
                    }
                    let part_name = i18n::t(Part::Battery.key());
                    let mut vars = std::collections::HashMap::new();
                    vars.insert("part", part_name.as_str());
                    set_status(&i18n::tr("vehicle.announce.no_spare", Some(&vars)));
                }
                3 => {
                    // Use Spare Alternator
                    if breakdown_part == Some(Part::Alternator) {
                        if let Some((_, _, alt_count, _)) = spare_counts {
                            if alt_count > 0 {
                                on_repair.emit(VehicleAction::UseSpare(Part::Alternator));
                                let part_name = i18n::t(Part::Alternator.key());
                                let mut vars = std::collections::HashMap::new();
                                vars.insert("part", part_name.as_str());
                                vars.insert("sup", "1");
                                set_status(&i18n::tr("vehicle.announce.used_spare", Some(&vars)));
                                return;
                            }
                        }
                    }
                    let part_name = i18n::t(Part::Alternator.key());
                    let mut vars = std::collections::HashMap::new();
                    vars.insert("part", part_name.as_str());
                    set_status(&i18n::tr("vehicle.announce.no_spare", Some(&vars)));
                }
                4 => {
                    // Use Spare Fuel Pump
                    if breakdown_part == Some(Part::FuelPump) {
                        if let Some((_, _, _, pump_count)) = spare_counts {
                            if pump_count > 0 {
                                on_repair.emit(VehicleAction::UseSpare(Part::FuelPump));
                                let part_name = i18n::t(Part::FuelPump.key());
                                let mut vars = std::collections::HashMap::new();
                                vars.insert("part", part_name.as_str());
                                vars.insert("sup", "1");
                                set_status(&i18n::tr("vehicle.announce.used_spare", Some(&vars)));
                                return;
                            }
                        }
                    }
                    let part_name = i18n::t(Part::FuelPump.key());
                    let mut vars = std::collections::HashMap::new();
                    vars.insert("part", part_name.as_str());
                    set_status(&i18n::tr("vehicle.announce.no_spare", Some(&vars)));
                }
                5 => {
                    // Hack Fix
                    if breakdown_part.is_some() {
                        on_repair.emit(VehicleAction::HackFix);
                        let mut vars = std::collections::HashMap::new();
                        vars.insert("sup", "3");
                        vars.insert("cred", "1");
                        vars.insert("day", "1");
                        set_status(&i18n::tr("vehicle.announce.hack_applied", Some(&vars)));
                    } else {
                        set_status(&i18n::t("vehicle.no_active"));
                    }
                }
                0 => {
                    // Back
                    on_back.emit(());
                    set_status(&i18n::t("menu.back"));
                }
                _ => {}
            }
        })
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
                let count = spare_counts.map(|(tire, _, _, _)| tire).unwrap_or(0);
                let enabled = breakdown_part == Some(Part::Tire) && count > 0;
                (
                    format!("{} (x{})", i18n::t("vehicle.spares.tire"), count),
                    enabled,
                )
            }),
            (2, {
                let count = spare_counts.map(|(_, battery, _, _)| battery).unwrap_or(0);
                let enabled = breakdown_part == Some(Part::Battery) && count > 0;
                (
                    format!("{} (x{})", i18n::t("vehicle.spares.battery"), count),
                    enabled,
                )
            }),
            (3, {
                let count = spare_counts.map(|(_, _, alt, _)| alt).unwrap_or(0);
                let enabled = breakdown_part == Some(Part::Alternator) && count > 0;
                (
                    format!("{} (x{})", i18n::t("vehicle.spares.alt"), count),
                    enabled,
                )
            }),
            (4, {
                let count = spare_counts.map(|(_, _, _, pump)| pump).unwrap_or(0);
                let enabled = breakdown_part == Some(Part::FuelPump) && count > 0;
                (
                    format!("{} (x{})", i18n::t("vehicle.spares.pump"), count),
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

    let setsize = items.len() as u8;

    // Status message
    let status_msg = if let Some(breakdown) = breakdown {
        let part_name = i18n::t(breakdown.part.key());
        let mut vars = std::collections::HashMap::new();
        vars.insert("part", part_name.as_str());
        i18n::tr("vehicle.breakdown", Some(&vars))
    } else {
        i18n::t("vehicle.no_active")
    };

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
