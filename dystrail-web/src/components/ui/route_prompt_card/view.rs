use super::interactions::{activate_handler, focus_effect, keydown_handler};
use super::option::RoutePromptOption;
use super::view_model::{RoutePromptViewModel, build_route_prompt_viewmodel};
use crate::game::OtDeluxeRoutePrompt;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct RoutePromptCardProps {
    pub prompt: OtDeluxeRoutePrompt,
    pub on_choice: Callback<crate::game::OtDeluxeRouteDecision>,
}

#[function_component(RoutePromptCard)]
pub fn route_prompt_card(props: &RoutePromptCardProps) -> Html {
    let focus_idx = use_state(|| 1_u8);
    let list_ref = use_node_ref();
    let resolved = use_state(|| false);

    let vm: RoutePromptViewModel = build_route_prompt_viewmodel(props.prompt);
    let activate = activate_handler(
        props.on_choice.clone(),
        resolved.clone(),
        vm.primary_decision,
        vm.secondary_decision,
    );

    focus_effect(list_ref.clone(), &focus_idx);
    let on_keydown = keydown_handler(activate.clone(), &focus_idx, &resolved);
    let setsize = 2_u8;

    html! {
        <section role="region"
                 aria-labelledby="route-prompt-title"
                 onkeydown={on_keydown}
                 class="ot-crossing">
            <h3 id="route-prompt-title">{ vm.title.clone() }</h3>
            <p class="muted">{ vm.prompt.clone() }</p>

            <ul role="menu" aria-label={vm.title.clone()} ref={list_ref}>
                <RoutePromptOption
                    index={1}
                    label={AttrValue::from(vm.primary_label)}
                    desc={AttrValue::from(vm.primary_desc)}
                    focused={*focus_idx == 1}
                    disabled={false}
                    posinset={1}
                    setsize={setsize}
                    on_activate={activate.clone()}
                />
                <RoutePromptOption
                    index={2}
                    label={AttrValue::from(vm.secondary_label)}
                    desc={AttrValue::from(vm.secondary_desc)}
                    focused={*focus_idx == 2}
                    disabled={false}
                    posinset={2}
                    setsize={setsize}
                    on_activate={activate}
                />
            </ul>
            <p aria-live="polite" class="muted status-line"></p>
        </section>
    }
}
