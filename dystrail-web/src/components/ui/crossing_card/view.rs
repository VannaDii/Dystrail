use super::interactions::{activate_handler, focus_effect, keydown_handler};
use super::option::CrossingOption;
use super::view_model::{CrossingViewModel, build_crossing_viewmodel};
use crate::game::{CrossingConfig, CrossingKind, GameState};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct CrossingCardProps {
    pub game_state: Rc<GameState>,
    pub config: Rc<CrossingConfig>,
    pub kind: CrossingKind,
    pub on_choice: Callback<u8>,
}

impl PartialEq for CrossingCardProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.game_state, &other.game_state)
            && Rc::ptr_eq(&self.config, &other.config)
            && self.kind == other.kind
    }
}

#[function_component(CrossingCard)]
pub fn crossing_card(props: &CrossingCardProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();
    let resolved = use_state(|| false);

    let vm: CrossingViewModel =
        match build_crossing_viewmodel(&props.game_state, &props.config, props.kind) {
            Ok(vm) => vm,
            Err(error_msg) => {
                return html! {
                    <section role="region" class="ot-crossing error">
                        <h3>{"Configuration Error"}</h3>
                        <p class="error">{ error_msg }</p>
                    </section>
                };
            }
        };

    let activate = activate_handler(
        props.on_choice.clone(),
        resolved.clone(),
        vm.bribe_available,
        vm.permit_available,
    );

    focus_effect(list_ref.clone(), &focus_idx);
    let on_keydown = keydown_handler(activate.clone(), &focus_idx, &resolved);
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
