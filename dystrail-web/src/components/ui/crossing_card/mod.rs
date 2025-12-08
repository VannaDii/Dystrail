mod option;
mod view_model;

use crate::a11y::set_status;
use crate::dom;
use crate::game::{CrossingConfig, CrossingKind, GameState};
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use option::CrossingOption;
use std::cell::RefCell;
use std::rc::Rc;
use view_model::{CrossingViewModel, apply_choice, build_crossing_viewmodel};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::KeyboardEvent;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct CrossingCardProps {
    pub game_state: Rc<RefCell<GameState>>,
    pub config: Rc<CrossingConfig>,
    pub kind: CrossingKind,
    pub on_resolved: Callback<()>,
}

impl PartialEq for CrossingCardProps {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.game_state.as_ptr(), other.game_state.as_ptr())
            && Rc::ptr_eq(&self.config, &other.config)
            && self.kind == other.kind
    }
}

#[function_component(CrossingCard)]
pub fn crossing_card(props: &CrossingCardProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();
    let resolved = use_state(|| false);

    let vm: CrossingViewModel = {
        let gs = props.game_state.borrow();
        match build_crossing_viewmodel(&gs, &props.config, props.kind) {
            Ok(vm) => vm,
            Err(error_msg) => {
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
                return;
            }

            let mut gs = game_state.borrow_mut();
            let result_msg = apply_choice(idx, &mut gs, &config, kind);
            drop(gs);

            if idx == 0 {
                on_resolved.emit(());
            }

            set_status(&result_msg);
            resolved.set(true);

            let on_resolved = on_resolved.clone();
            let timeout = Closure::once(move || {
                on_resolved.emit(());
            });
            if let Some(win) = dom::window() {
                if let Err(err) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    timeout.as_ref().unchecked_ref(),
                    1000,
                ) {
                    dom::console_error(&format!(
                        "Failed to delay crossing transition: {}",
                        dom::js_error_message(&err)
                    ));
                }
            } else {
                dom::console_error("Failed to delay crossing transition: window unavailable");
            }
            timeout.forget();
        })
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
        let resolved = *resolved;
        Callback::from(move |e: KeyboardEvent| {
            if resolved {
                return;
            }

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
                let next = match *focus_idx {
                    1 => 2,
                    2 => 3,
                    3 => 0,
                    _ => 1,
                };
                focus_idx.set(next);
                e.prevent_default();
            } else if key == "ArrowUp" {
                let prev = match *focus_idx {
                    0 => 3,
                    1 => 0,
                    3 => 2,
                    _ => 1,
                };
                focus_idx.set(prev);
                e.prevent_default();
            }
        })
    };

    let setsize = 4_u8;

    html! {
        <section role="region"
                 aria-labelledby="cross-title"
                 onkeydown={on_keydown}
                 class="ot-crossing">
            <h3 id="cross-title">{ vm.title.clone() }</h3>
            <p class="muted">{ vm.prompt.clone() }</p>

            { vm.shutdown_notice.clone().map_or_else(
                || html! {},
                |notice| html! { <p class="warning" aria-live="polite">{ notice }</p> },
            ) }

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
