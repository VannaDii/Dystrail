use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

use crate::a11y::set_status;
use crate::game::{
    CrossingConfig, CrossingKind, GameState, apply_bribe, apply_detour, apply_permit,
    calculate_bribe_cost, can_afford_bribe, can_use_permit,
};
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use gloo::timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct CrossingViewModel {
    pub title: String,
    pub prompt: String,
    pub detour_label: String,
    pub detour_desc: String,
    pub bribe_label: String,
    pub bribe_desc: String,
    pub permit_label: String,
    pub permit_desc: String,
    pub back_label: String,
    pub permit_available: bool,
    pub bribe_available: bool,
    pub shutdown_notice: Option<String>,
}

#[derive(Properties, PartialEq, Clone)]
pub struct CrossingOptionProps {
    pub index: u8, // 1, 2, 3, or 0
    pub label: AttrValue,
    pub desc: AttrValue,
    pub focused: bool,
    pub disabled: bool,
    pub posinset: u8,
    pub setsize: u8,
    pub on_activate: Callback<u8>,
}

#[function_component(CrossingOption)]
pub fn crossing_option(p: &CrossingOptionProps) -> Html {
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

    let desc_id = format!("desc-{idx}");

    html! {
        <li role="menuitem"
            tabindex={ if p.focused { "0" } else { "-1" } }
            data-key={idx.to_string()}
            aria-posinset={p.posinset.to_string()}
            aria-setsize={p.setsize.to_string()}
            aria-describedby={desc_id.clone()}
            aria-disabled={ if p.disabled { "true" } else { "false" } }
            onclick={on_click}
            class={ classes!("ot-menuitem", if p.disabled { Some("disabled") } else { None }) }>
            <span class="num">{ format!("{idx})") }</span>
            <span class="label">{ p.label.clone() }</span>
            <small id={desc_id} class="muted desc">{ p.desc.clone() }</small>
        </li>
    }
}

#[derive(Properties, Clone)]
pub struct CrossingCardProps {
    pub game_state: Rc<RefCell<GameState>>,
    pub config: Rc<CrossingConfig>,
    pub kind: CrossingKind,
    pub on_resolved: Callback<()>,
}

impl PartialEq for CrossingCardProps {
    fn eq(&self, other: &Self) -> bool {
        // For functional components with RefCell, we can compare the pointer addresses
        // or skip deep comparison since props changes should trigger re-renders anyway
        std::ptr::eq(self.game_state.as_ptr(), other.game_state.as_ptr())
            && Rc::ptr_eq(&self.config, &other.config)
            && self.kind == other.kind
    }
}

/// Build view model with resolved strings and availability flags
fn build_crossing_viewmodel(
    gs: &GameState,
    cfg: &CrossingConfig,
    kind: CrossingKind,
) -> Result<CrossingViewModel, String> {
    let type_cfg = cfg
        .types
        .get(&kind)
        .ok_or_else(|| format!("Unknown crossing type: {kind:?}"))?;

    // Base title and prompt
    let title = match kind {
        CrossingKind::Checkpoint => i18n::t("cross.types.checkpoint"),
        CrossingKind::BridgeOut => i18n::t("cross.types.bridge_out"),
    };
    let prompt = i18n::t("cross.prompt");

    // Calculate costs and effects with modifiers
    let mut detour_days = type_cfg.detour.days;
    let mut detour_pants = type_cfg.detour.pants;

    // Apply weather modifiers for detour
    if let Some(weather_mod) = cfg.global_mods.weather.get(&gs.weather_state.today) {
        if let Some(extra_days) = weather_mod.detour.days {
            detour_days += extra_days;
        }
        if let Some(extra_pants) = weather_mod.detour.pants {
            detour_pants += extra_pants;
        }
    }

    // Format detour with signed deltas
    let days_str = if detour_days >= 0 {
        format!("+{detour_days}")
    } else {
        detour_days.to_string()
    };
    let supplies_str = if type_cfg.detour.supplies >= 0 {
        format!("+{supplies}", supplies = type_cfg.detour.supplies)
    } else {
        type_cfg.detour.supplies.to_string()
    };
    let pants_str = if detour_pants >= 0 {
        format!("+{detour_pants}")
    } else {
        detour_pants.to_string()
    };

    let mut detour_args = std::collections::HashMap::new();
    detour_args.insert("days", days_str.as_str());
    detour_args.insert("supplies", supplies_str.as_str());
    detour_args.insert("pants", pants_str.as_str());
    let detour_label = i18n::tr("cross.options.detour", Some(&detour_args));
    let detour_desc = i18n::t("cross.desc.detour");

    // Calculate bribe cost with persona discount
    let bribe_cost_cents =
        calculate_bribe_cost(type_cfg.bribe.base_cost_cents, gs.mods.bribe_discount_pct);
    let bribe_cost_display = format_currency(bribe_cost_cents);
    let mut bribe_args = std::collections::HashMap::new();
    bribe_args.insert("cost", bribe_cost_display.as_str());
    let bribe_label = i18n::tr("cross.options.bribe", Some(&bribe_args));
    let bribe_desc = i18n::t("cross.desc.bribe");

    // Permit
    let permit_label = i18n::t("cross.options.permit");
    let permit_desc = i18n::t("cross.desc.permit");

    // Back
    let back_label = i18n::t("cross.options.back");

    // Availability checks
    let permit_available = can_use_permit(gs, &kind);
    let bribe_available = can_afford_bribe(gs, &kind);

    // Shutdown notice for bribe if active
    let shutdown_notice = if let Some(exec_mod) = cfg.global_mods.exec_orders.get("Shutdown") {
        // Check if current exec order is Shutdown
        let order_name = format!("{:?}", gs.current_order);
        if order_name == "Shutdown" {
            #[allow(clippy::cast_possible_truncation)]
            let chance_pct = (exec_mod.bribe_success_chance * 100.0) as i32;
            let mut args = std::collections::HashMap::new();
            let chance_str = chance_pct.to_string();
            args.insert("chance", chance_str.as_str());
            Some(i18n::tr("cross.policy.shutdown", Some(&args)))
        } else {
            None
        }
    } else {
        None
    };

    Ok(CrossingViewModel {
        title,
        prompt,
        detour_label,
        detour_desc,
        bribe_label,
        bribe_desc,
        permit_label,
        permit_desc,
        back_label,
        permit_available,
        bribe_available,
        shutdown_notice,
    })
}

fn format_currency(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let cents_abs = cents.unsigned_abs();
    let dollars = cents_abs / 100;
    let remainder = cents_abs % 100;
    format!("{sign}${dollars}.{remainder:02}")
}

#[function_component(CrossingCard)]
pub fn crossing_card(props: &CrossingCardProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();
    let resolved = use_state(|| false);

    // Build view model with safe error handling
    let vm = {
        let gs = props.game_state.borrow();
        match build_crossing_viewmodel(&gs, &props.config, props.kind) {
            Ok(vm) => vm,
            Err(error_msg) => {
                // Return error UI instead of panicking
                return html! {
                    <section role="region" class="ot-crossing error">
                        <h3>{"Configuration Error"}</h3>
                        <p class="error">{ error_msg }</p>
                    </section>
                };
            }
        }
    };

    let activate = {
        let game_state = props.game_state.clone();
        let config = props.config.clone();
        let kind = props.kind;
        let on_resolved = props.on_resolved.clone();
        let resolved = resolved.clone();
        Callback::from(move |idx: u8| {
            if *resolved {
                return; // Already resolved, ignore
            }

            let result_msg = match idx {
                1 => {
                    // Detour
                    let mut gs = game_state.borrow_mut();
                    apply_detour(&mut gs, &config, kind)
                }
                2 => {
                    // Bribe
                    let mut gs = game_state.borrow_mut();
                    apply_bribe(&mut gs, &config, kind)
                }
                3 => {
                    // Permit
                    let mut gs = game_state.borrow_mut();
                    apply_permit(&mut gs, &config, kind)
                }
                0 => {
                    // Back - only if it's a preview/non-blocking encounter
                    // For blocking crossings, this should be disabled or no-op
                    return; // TODO: Implement proper back logic based on encounter context
                }
                _ => return,
            };

            // Set status and mark as resolved
            set_status(&result_msg);
            resolved.set(true);

            // Call resolution callback after a brief delay
            let on_resolved = on_resolved.clone();
            spawn_local(async move {
                TimeoutFuture::new(1000).await;
                on_resolved.emit(());
            });
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
        let resolved = *resolved;
        Callback::from(move |e: KeyboardEvent| {
            if resolved {
                return; // Ignore input after resolution
            }

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
                // Could close or go back depending on context
                activate.emit(0);
                e.prevent_default();
            } else if key == "ArrowDown" {
                let next = match *focus_idx {
                    1 => 2,
                    2 => 3,
                    3 => 0,
                    _ => 1, // 0 or any other value goes to 1
                };
                focus_idx.set(next);
                e.prevent_default();
            } else if key == "ArrowUp" {
                let prev = match *focus_idx {
                    0 => 3,
                    1 => 0,
                    3 => 2,
                    _ => 1, // 2 or any other value goes to 1
                };
                focus_idx.set(prev);
                e.prevent_default();
            }
        })
    };

    let setsize = 4_u8; // 1, 2, 3, 0

    html! {
        <section role="region"
                 aria-labelledby="cross-title"
                 onkeydown={on_keydown}
                 class="ot-crossing">
            <h3 id="cross-title">{ vm.title.clone() }</h3>
            <p class="muted">{ vm.prompt.clone() }</p>

            { if let Some(notice) = vm.shutdown_notice.clone() {
                html! { <p class="warning" aria-live="polite">{ notice }</p> }
            } else {
                html! {}
            }}

            <ul role="menu" aria-label={vm.title.clone()} ref={list_ref}>
                <CrossingOption
                    index={1}
                    label={AttrValue::from(vm.detour_label)}
                    desc={AttrValue::from(vm.detour_desc)}
                    focused={*focus_idx == 1}
                    disabled={false}
                    posinset={1}
                    setsize={setsize}
                    on_activate={activate.clone()}
                />
                <CrossingOption
                    index={2}
                    label={AttrValue::from(vm.bribe_label)}
                    desc={AttrValue::from(vm.bribe_desc)}
                    focused={*focus_idx == 2}
                    disabled={!vm.bribe_available}
                    posinset={2}
                    setsize={setsize}
                    on_activate={activate.clone()}
                />
                <CrossingOption
                    index={3}
                    label={AttrValue::from(vm.permit_label)}
                    desc={AttrValue::from(vm.permit_desc)}
                    focused={*focus_idx == 3}
                    disabled={!vm.permit_available}
                    posinset={3}
                    setsize={setsize}
                    on_activate={activate.clone()}
                />
                <CrossingOption
                    index={0}
                    label={AttrValue::from(vm.back_label)}
                    desc={AttrValue::from("")}
                    focused={*focus_idx == 0}
                    disabled={false}
                    posinset={4}
                    setsize={setsize}
                    on_activate={activate}
                />
            </ul>
            <p aria-live="polite" class="muted status-line"></p>
        </section>
    }
}
