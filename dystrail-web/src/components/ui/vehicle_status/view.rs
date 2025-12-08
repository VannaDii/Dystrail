use super::logic::{SelectionResolution, VehicleAction, evaluate_selection};
use super::menu_item::VehicleMenuItem;
use crate::a11y::set_status;
use crate::game::{GameState, Part};
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct VehicleStatusProps {
    pub game_state: Option<Rc<GameState>>,
    pub on_back: Callback<()>,
    pub on_repair_action: Callback<VehicleAction>,
}

impl PartialEq for VehicleStatusProps {
    fn eq(&self, other: &Self) -> bool {
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
    };

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

    let setsize = u8::try_from(items.len()).unwrap_or(u8::MAX);

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
              let posinset = u8::try_from(i).unwrap_or_default().saturating_add(1);
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
