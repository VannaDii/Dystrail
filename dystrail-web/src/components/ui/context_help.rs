//! Viewport-contained help rendered above scene and panel clipping boundaries.
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct Props {
    pub title: String,
    #[prop_or_default]
    pub text: String,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_else(|| "?".to_owned())]
    pub icon: String,
    #[prop_or_default]
    pub graphic: Option<Html>,
    #[prop_or_default]
    pub source_href: Option<String>,
    #[prop_or_default]
    pub trigger_label: Option<String>,
    #[prop_or_default]
    pub informational: bool,
}
#[function_component(ContextHelp)]
pub fn context_help(p: &Props) -> Html {
    crate::i18n::use_language();
    let open = use_state(|| false);
    let preference = use_context::<crate::app::help::HelpPreference>().unwrap_or_default();
    let visible = p.informational || preference.0;
    let expanded = visible && *open;
    {
        let open = open.clone();
        use_effect_with(visible, move |visible| {
            if !visible {
                open.set(false);
            }
        });
    }
    let node = use_node_ref();
    let panel = use_node_ref();
    let position = use_popup_position(
        node.clone(),
        panel.clone(),
        expanded,
        (p.title.clone(), p.text.clone(), p.children.clone()),
    );
    let dismiss = {
        let open = open.clone();
        let node = node.clone();
        let panel = panel.clone();
        Callback::from(move |()| {
            if contains_focus(&panel) {
                focus_first_button(&node);
            }
            open.set(false);
        })
    };
    super::dismiss::use_outside_dismiss(node.clone(), expanded, dismiss);
    let close = {
        let open = open.clone();
        let node = node.clone();
        Callback::from(move |_| {
            open.set(false);
            focus_first_button(&node);
        })
    };
    let toggle = {
        let position = position.clone();
        Callback::from(move |_| {
            position.set(String::new());
            open.set(!*open);
        })
    };
    let enter_popup = {
        let panel = panel.clone();
        Callback::from(move |e: KeyboardEvent| {
            if expanded && e.key() == "Tab" && !e.shift_key() {
                e.prevent_default();
                focus_first_button(&panel);
            }
        })
    };
    let popup = if expanded {
        web_sys::window().and_then(|w|w.document()).and_then(|d|d.body()).map_or_else(Html::default,|body|yew::create_portal(html!{
            <div class="help-popover viewport-help" ref={panel} style={(*position).clone()} role="note" onpointerdown={Callback::from(|e:PointerEvent|e.stop_propagation())}>
                <div class="help-heading"><strong>{&p.title}</strong><button class="help-dismiss" type="button" aria-label={crate::i18n::t("dialogs.close")} onclick={close}><svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="m6 6 12 12M6 18 18 6" /></svg></button></div>
                if p.children.is_empty() {<span>{&p.text}</span>} else {{p.children.clone()}}
                if let Some(source)=&p.source_href {<a href={source.clone()} target="_blank" rel="noopener noreferrer">{crate::i18n::t("trail.source")}</a>}
            </div>
        },body.into()))
    } else {
        Html::default()
    };
    let label = p
        .trigger_label
        .clone()
        .unwrap_or_else(|| format!("{}: {}", crate::i18n::t("play.help"), p.title));
    let glyph = p.graphic.clone().unwrap_or_else(|| {
        if p.trigger_label.is_none() && matches!(p.icon.as_str(), "ⓘ" | "i") {
            html! {<span class="info-glyph" aria-hidden="true">{"i"}</span>}
        } else {
            html! {p.trigger_label.as_ref().unwrap_or(&p.icon)}
        }
    });
    html! {<span class={classes!("context-help",(!visible).then_some("context-help-hidden"))} data-help-kind={if p.informational {"info"}else{"tip"}} aria-hidden={(!visible).then_some("true")} ref={node}>
        <button type="button" disabled={!visible} class={classes!("help-trigger",(p.trigger_label.is_some() && p.graphic.is_none()).then_some("help-trigger-labeled"))} aria-label={label} aria-expanded={expanded.to_string()} onclick={toggle} onkeydown={enter_popup}>{glyph}</button>
        {popup}
    </span>}
}

#[hook]
fn use_popup_position(
    node: NodeRef,
    panel: NodeRef,
    expanded: bool,
    content: (String, String, Children),
) -> UseStateHandle<String> {
    let position = use_state(String::new);
    {
        let position = position.clone();
        use_effect_with((expanded, content), move |(open, _)| {
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
                    (targets, callback)
                })
            } else {
                None
            };
            move || {
                if let Some((targets, callback)) = listener {
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
    position
}

fn popup_position(node: &NodeRef, panel: &NodeRef) -> Option<String> {
    let window = web_sys::window()?;
    let trigger = node.cast::<web_sys::Element>()?.get_bounding_client_rect();
    let (width, height, offset_x, offset_y) = if let Some(viewport) = window.visual_viewport() {
        (
            viewport.width(),
            viewport.height(),
            viewport.offset_left(),
            viewport.offset_top(),
        )
    } else {
        (
            window.inner_width().ok()?.as_f64()?,
            window.inner_height().ok()?.as_f64()?,
            0.0,
            0.0,
        )
    };
    let max_width = (width - 24.0).max(1.0);
    let max_height = (height - 24.0).max(1.0);
    let element = panel.cast::<web_sys::HtmlElement>()?;
    let _ = element
        .style()
        .set_property("max-width", &format!("{max_width}px"));
    let _ = element
        .style()
        .set_property("max-height", &format!("{max_height}px"));
    let popup = element.get_bounding_client_rect();
    let left = (trigger.left() + (trigger.width() - popup.width()) / 2.0).clamp(
        offset_x + 12.0,
        (offset_x + width - popup.width() - 12.0).max(offset_x + 12.0),
    );
    let top = if trigger.bottom() + 10.0 + popup.height() <= offset_y + height - 12.0 {
        trigger.bottom() + 10.0
    } else {
        (trigger.top() - popup.height() - 10.0).max(offset_y + 12.0)
    };
    let top = top.clamp(
        offset_y + 12.0,
        (offset_y + height - popup.height() - 12.0).max(offset_y + 12.0),
    );
    Some(format!(
        "left:{left}px;top:{top}px;max-width:{max_width}px;max-height:{max_height}px;visibility:visible"
    ))
}

fn contains_focus(node: &NodeRef) -> bool {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element())
        .is_some_and(|active| {
            node.cast::<web_sys::Node>()
                .is_some_and(|root| root.contains(Some(&active)))
        })
}

fn focus_first_button(node: &NodeRef) {
    if let Some(button) = node
        .cast::<web_sys::Element>()
        .and_then(|element| element.query_selector("button").ok().flatten())
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = button.focus();
    }
}
