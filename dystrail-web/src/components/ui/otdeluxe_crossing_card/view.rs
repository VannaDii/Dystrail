use super::interactions::{activate_handler, focus_effect, keydown_handler};
use super::option::OtDeluxeCrossingOption;
use super::view_model::{OtDeluxeCrossingViewModel, build_otdeluxe_crossing_viewmodel};
use crate::game::GameState;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct OtDeluxeCrossingCardProps {
    pub game_state: Rc<GameState>,
    pub on_choice: Callback<u8>,
}

impl PartialEq for OtDeluxeCrossingCardProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.game_state, &other.game_state)
    }
}

#[function_component(OtDeluxeCrossingCard)]
pub fn otdeluxe_crossing_card(props: &OtDeluxeCrossingCardProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();
    let resolved = use_state(|| false);

    let vm: OtDeluxeCrossingViewModel = match build_otdeluxe_crossing_viewmodel(&props.game_state) {
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

    let activate = activate_handler(props.on_choice.clone(), resolved.clone(), vm.options);

    focus_effect(list_ref.clone(), &focus_idx);
    let on_keydown = keydown_handler(activate.clone(), &focus_idx, &resolved);
    let setsize = 5_u8;

    html! {
        <section role="region"
                 aria-labelledby="ot-crossing-title"
                 onkeydown={on_keydown}
                 class="ot-crossing">
            <h3 id="ot-crossing-title">{ vm.title.clone() }</h3>
            <p class="muted">{ vm.prompt.clone() }</p>
            <p class="muted">{ vm.stats.clone() }</p>

            <ul role="menu" aria-label={vm.title.clone()} ref={list_ref}>
                <OtDeluxeCrossingOption
                    index={1}
                    label={AttrValue::from(vm.ford_label)}
                    desc={AttrValue::from(vm.ford_desc)}
                    focused={*focus_idx == 1}
                    disabled={!vm.options.ford()}
                    posinset={1}
                    setsize={setsize}
                    on_activate={activate.clone()}
                />
                <OtDeluxeCrossingOption
                    index={2}
                    label={AttrValue::from(vm.caulk_label)}
                    desc={AttrValue::from(vm.caulk_desc)}
                    focused={*focus_idx == 2}
                    disabled={!vm.options.caulk_float()}
                    posinset={2}
                    setsize={setsize}
                    on_activate={activate.clone()}
                />
                <OtDeluxeCrossingOption
                    index={3}
                    label={AttrValue::from(vm.ferry_label)}
                    desc={AttrValue::from(vm.ferry_desc)}
                    focused={*focus_idx == 3}
                    disabled={!vm.options.ferry()}
                    posinset={3}
                    setsize={setsize}
                    on_activate={activate.clone()}
                />
                <OtDeluxeCrossingOption
                    index={4}
                    label={AttrValue::from(vm.guide_label)}
                    desc={AttrValue::from(vm.guide_desc)}
                    focused={*focus_idx == 4}
                    disabled={!vm.options.guide()}
                    posinset={4}
                    setsize={setsize}
                    on_activate={activate.clone()}
                />
                <OtDeluxeCrossingOption
                    index={0}
                    label={AttrValue::from(vm.back_label)}
                    desc={AttrValue::from("")}
                    focused={*focus_idx == 0}
                    disabled={false}
                    posinset={5}
                    setsize={setsize}
                    on_activate={activate}
                />
            </ul>
            <p aria-live="polite" class="muted status-line"></p>
        </section>
    }
}
