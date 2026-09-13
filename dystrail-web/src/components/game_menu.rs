//! Compact top-level controls with click-away and keyboard dismissal.
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
}
#[function_component(GameMenu)]
pub fn game_menu(p: &Props) -> Html {
    crate::i18n::use_language();
    let open = use_state(|| false);
    let node = use_node_ref();
    let panel = use_node_ref();
    use_menu_bounds(node.clone(), panel.clone(), *open, p.children.clone());
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
        if *open {<div id="game-menu-panel" class="top-game-menu-panel" ref={panel} onclick={select}>{for p.children.iter()}</div>}
    </div>}
}

#[hook]
fn use_menu_bounds(node: NodeRef, panel: NodeRef, expanded: bool, content: Children) {
    use_effect_with((expanded, content), move |(open, _)| {
        let tracking = if *open && cfg!(target_arch = "wasm32") {
            web_sys::window().map(|window| {
                let place = move || constrain_menu_height(&panel);
                place();
                let callback = Closure::wrap(Box::new(place) as Box<dyn FnMut()>);
                let observer = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()).ok();
                if let Some(observer) = &observer
                    && let Some(trigger) = node.cast::<web_sys::Element>()
                {
                    observer.observe(&trigger);
                    if let Ok(Some(header)) = trigger.closest(".game-header") {
                        observer.observe(&header);
                    }
                }
                let mut targets = vec![window.clone().unchecked_into::<web_sys::EventTarget>()];
                if let Some(viewport) = window.visual_viewport() {
                    targets.push(viewport.unchecked_into());
                }
                for target in &targets {
                    for event in ["resize", "scroll"] {
                        let _ = target.add_event_listener_with_callback_and_bool(
                            event,
                            callback.as_ref().unchecked_ref(),
                            true,
                        );
                    }
                }
                (targets, callback, observer)
            })
        } else {
            None
        };
        move || {
            if let Some((targets, callback, observer)) = tracking {
                if let Some(observer) = observer {
                    observer.disconnect();
                }
                for target in targets {
                    for event in ["resize", "scroll"] {
                        let _ = target.remove_event_listener_with_callback_and_bool(
                            event,
                            callback.as_ref().unchecked_ref(),
                            true,
                        );
                    }
                }
            }
        }
    });
}

fn constrain_menu_height(panel: &NodeRef) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(panel) = panel.cast::<web_sys::HtmlElement>() else {
        return;
    };
    let Some(bottom) = window
        .visual_viewport()
        .map(|viewport| viewport.offset_top() + viewport.height())
        .or_else(|| {
            window
                .inner_height()
                .ok()
                .and_then(|height| height.as_f64())
        })
    else {
        return;
    };
    let available = (bottom - panel.get_bounding_client_rect().top() - 12.0).max(1.0);
    let limit = format!("{available}px");
    let style = panel.style();
    if style
        .get_property_value("--menu-max-height")
        .ok()
        .as_deref()
        == Some(limit.as_str())
    {
        return;
    }
    let _ = style.set_property("--menu-max-height", &limit);
    if let Some(active) = window
        .document()
        .and_then(|document| document.active_element())
        && panel.contains(Some(&active))
    {
        let top = panel.get_bounding_client_rect().top() + f64::from(panel.client_top());
        let bottom = top + f64::from(panel.client_height());
        let focused = active.get_bounding_client_rect();
        if focused.top() < top {
            panel.scroll_by_with_x_and_y(0.0, (focused.top() - top).floor());
        } else if focused.bottom() > bottom {
            panel.scroll_by_with_x_and_y(0.0, (focused.bottom() - bottom).ceil());
        }
    }
}
