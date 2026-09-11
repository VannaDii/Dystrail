//! Viewport-contained help rendered above scene and panel clipping boundaries.
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;
#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    pub title: String,
    pub text: String,
    #[prop_or_else(|| "?".to_owned())]
    pub icon: String,
    #[prop_or_default]
    pub source_href: Option<String>,
}
#[function_component(ContextHelp)]
pub fn context_help(p: &Props) -> Html {
    let open = use_state(|| false);
    let node = use_node_ref();
    let panel = use_node_ref();
    let position = use_state(String::new);
    let dismiss = {
        let open = open.clone();
        Callback::from(move |()| open.set(false))
    };
    super::dismiss::use_outside_dismiss(node.clone(), *open, dismiss);
    {
        let node = node.clone();
        let panel = panel.clone();
        let position = position.clone();
        use_effect_with(*open, move |open| {
            let listener = if *open {
                web_sys::window().map(|window| {
                    let place = move || {
                        if let Some(style) = popup_position(&node, &panel) {
                            position.set(style);
                        }
                    };
                    place();
                    let callback = Closure::wrap(Box::new(move |_e: web_sys::Event| place())
                        as Box<dyn FnMut(web_sys::Event)>);
                    for event in ["resize", "scroll"] {
                        let _ = window.add_event_listener_with_callback_and_bool(
                            event,
                            callback.as_ref().unchecked_ref(),
                            true,
                        );
                    }
                    (window, callback)
                })
            } else {
                None
            };
            move || {
                if let Some((window, callback)) = listener {
                    for event in ["resize", "scroll"] {
                        let _ = window.remove_event_listener_with_callback_and_bool(
                            event,
                            callback.as_ref().unchecked_ref(),
                            true,
                        );
                    }
                }
            }
        });
    }
    let toggle = {
        let open = open.clone();
        let position = position.clone();
        Callback::from(move |_| {
            position.set(String::new());
            open.set(!*open);
        })
    };
    let popup = if *open {
        web_sys::window().and_then(|w|w.document()).and_then(|d|d.body()).map_or_else(Html::default,|body|yew::create_portal(html!{
            <span class="help-popover viewport-help" ref={panel} style={(*position).clone()} role="note" onpointerdown={Callback::from(|e:PointerEvent|e.stop_propagation())}><strong>{&p.title}</strong><span>{&p.text}</span>
                if let Some(source)=&p.source_href {<a href={source.clone()} target="_blank" rel="noopener noreferrer">{crate::i18n::t("trail.source")}</a>}
            </span>
        },body.into()))
    } else {
        Html::default()
    };
    html! {<span class="context-help" ref={node}>
        <button type="button" class="help-trigger" aria-label={format!("{}: {}",crate::i18n::t("play.help"),p.title)} aria-expanded={open.to_string()} onclick={toggle}>{&p.icon}</button>
        {popup}
    </span>}
}

fn popup_position(node: &NodeRef, panel: &NodeRef) -> Option<String> {
    let window = web_sys::window()?;
    let trigger = node.cast::<web_sys::Element>()?.get_bounding_client_rect();
    let popup = panel.cast::<web_sys::Element>()?.get_bounding_client_rect();
    let width = window.inner_width().ok()?.as_f64()?;
    let height = window.inner_height().ok()?.as_f64()?;
    let left = (trigger.left() + (trigger.width() - popup.width()) / 2.0)
        .clamp(12.0, (width - popup.width() - 12.0).max(12.0));
    let top = if trigger.bottom() + 10.0 + popup.height() <= height - 12.0 {
        trigger.bottom() + 10.0
    } else {
        (trigger.top() - popup.height() - 10.0).max(12.0)
    };
    let top = top.clamp(12.0, (height - popup.height() - 12.0).max(12.0));
    Some(format!("left:{left}px;top:{top}px;visibility:visible"))
}
