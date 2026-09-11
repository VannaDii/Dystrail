//! Dismiss overlays without moving focus; activate numbered choices from anywhere on the page.
use wasm_bindgen::{JsCast, closure::Closure};
use yew::prelude::*;
#[hook]
pub fn use_outside_dismiss(node: NodeRef, open: bool, close: Callback<()>) {
    use_effect_with((open, node, close), move |(open, node, close)| {
        let listener = if *open && cfg!(target_arch = "wasm32") {
            web_sys::window().and_then(|w| w.document()).map(|doc| {
                let node = node.clone();
                let close = close.clone();
                let handler = Closure::wrap(Box::new(move |event: web_sys::Event| {
                    let outside = event.type_() == "pointerdown"
                        && event
                            .target()
                            .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
                            .is_some_and(|target| {
                                node.cast::<web_sys::Node>()
                                    .is_some_and(|root| !root.contains(Some(&target)))
                            });
                    let escape = event
                        .dyn_ref::<web_sys::KeyboardEvent>()
                        .is_some_and(|key| key.key() == "Escape");
                    if outside || escape {
                        close.emit(());
                    }
                }) as Box<dyn FnMut(web_sys::Event)>);
                for name in ["pointerdown", "keydown"] {
                    let _ = doc
                        .add_event_listener_with_callback(name, handler.as_ref().unchecked_ref());
                }
                (doc, handler)
            })
        } else {
            None
        };
        move || {
            if let Some((doc, handler)) = listener {
                for name in ["pointerdown", "keydown"] {
                    let _ = doc.remove_event_listener_with_callback(
                        name,
                        handler.as_ref().unchecked_ref(),
                    );
                }
            }
        }
    });
}
#[hook]
pub fn use_number_shortcuts() {
    use_effect_with((), move |()| {
        let listener = if cfg!(target_arch = "wasm32") {
            web_sys::window().and_then(|w|w.document()).map(|doc| {
            let document=doc.clone();
            let handler=Closure::wrap(Box::new(move |event:web_sys::KeyboardEvent| {
                if event.default_prevented() || event.repeat() || event.ctrl_key() || event.alt_key() || event.meta_key() {return;}
                let key=event.key();if key.len()!=1 || !key.as_bytes()[0].is_ascii_digit(){return;}
                if event.target().and_then(|t|t.dyn_into::<web_sys::Element>().ok()).is_some_and(|el|el.closest("input,textarea,select,[contenteditable=true],[role=dialog][aria-modal=true]").ok().flatten().is_some()){return;}
                if document.query_selector("[role=dialog][aria-modal=true]").ok().flatten().is_some(){return;}
                let selector=format!("main [aria-keyshortcuts='{key}'],main [data-key='{key}'],main [role=menuitem][data-index='{key}']");
                if let Ok(nodes)=document.query_selector_all(&selector) {for i in 0..nodes.length(){
                    if let Some(el)=nodes.get(i).and_then(|n|n.dyn_into::<web_sys::HtmlElement>().ok()) && el.offset_width()>0 && !el.has_attribute("disabled") {event.prevent_default();el.click();break;}
                }}
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
            let _=doc.add_event_listener_with_callback("keydown",handler.as_ref().unchecked_ref());(doc,handler)
        })
        } else {
            None
        };
        move || {
            if let Some((doc, handler)) = listener {
                let _ = doc.remove_event_listener_with_callback(
                    "keydown",
                    handler.as_ref().unchecked_ref(),
                );
            }
        }
    });
}
