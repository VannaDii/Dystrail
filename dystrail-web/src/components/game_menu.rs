//! Compact top-level controls with click-away and keyboard dismissal.
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}
#[function_component(GameMenu)]
pub fn game_menu(p: &Props) -> Html {
    let open = use_state(|| false);
    let node = use_node_ref();
    let close = {
        let open = open.clone();
        Callback::from(move |()| open.set(false))
    };
    crate::components::ui::dismiss::use_outside_dismiss(node.clone(), *open, close);
    let toggle = {
        let open = open.clone();
        Callback::from(move |_| open.set(!*open))
    };
    let select = {
        let open = open.clone();
        Callback::from(move |e: MouseEvent| {
            use wasm_bindgen::JsCast;
            if e.target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|e| e.closest("[data-menu-close]").ok().flatten().is_some())
            {
                open.set(false);
            }
        })
    };
    let escape = {
        let open = open.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                open.set(false);
                crate::a11y::restore_focus("game-menu-button");
            }
        })
    };
    html! {<div class="top-game-menu" ref={node} onkeydown={escape}>
        <button id="game-menu-button" aria-expanded={open.to_string()} aria-controls="game-menu-panel" onclick={toggle}>{crate::i18n::t("ux.menu")}<span aria-hidden="true">{" ▾"}</span></button>
        if *open {<div id="game-menu-panel" class="top-game-menu-panel" onclick={select}>{for p.children.iter()}</div>}
    </div>}
}
