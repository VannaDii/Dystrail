//! Keep the language list in the browser's top layer, anchored to its trigger.
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;

#[hook]
pub(super) fn use_language_popover(node: NodeRef, panel: NodeRef, expanded: bool) {
    use_effect_with(expanded, move |open| {
        let tracking = if *open && cfg!(target_arch = "wasm32") {
            web_sys::window().and_then(|window| {
                let popup = panel.cast::<web_sys::HtmlElement>()?;
                popup.show_popover().ok()?;
                let trigger = node.cast::<web_sys::Element>()?;
                let place = move || position_list(&node, &panel);
                place();
                let callback = Closure::wrap(Box::new(place) as Box<dyn FnMut()>);
                let observer = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()).ok();
                if let Some(observer) = &observer {
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
                Some((targets, callback, observer))
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

fn position_list(node: &NodeRef, panel: &NodeRef) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(trigger) = node.cast::<web_sys::Element>() else {
        return;
    };
    let Some(panel) = panel.cast::<web_sys::HtmlElement>() else {
        return;
    };
    let Some(viewport) = window.visual_viewport() else {
        return;
    };
    let rect = trigger.get_bounding_client_rect();
    let edge = 12.0;
    let gap = 6.0;
    let left_edge = viewport.offset_left() + edge;
    let top_edge = viewport.offset_top() + edge;
    let bottom_edge = viewport.offset_top() + viewport.height() - edge;
    let width = rect.width().min((viewport.width() - edge * 2.0).max(1.0));
    let left = rect
        .left()
        .max(left_edge)
        .min((viewport.offset_left() + viewport.width() - edge - width).max(left_edge));
    let below = (bottom_edge - rect.bottom() - gap).max(1.0);
    let above = (rect.top() - gap - top_edge).max(1.0);
    let upwards = below < 144.0 && above > below;
    let height = if upwards { above } else { below }.min(320.0);
    let top = if upwards {
        rect.top() - gap - height
    } else {
        rect.bottom() + gap
    }
    .max(top_edge);
    let style = panel.style();
    for (property, value) in [
        ("left", left),
        ("top", top),
        ("width", width),
        ("max-height", height),
    ] {
        let _ = style.set_property(property, &format!("{value}px"));
    }
}
