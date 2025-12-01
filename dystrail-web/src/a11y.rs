// Accessibility helpers
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// Get CSS for visible focus indicators and screen reader utilities
///
/// Returns critical accessibility CSS that should be injected early in the page load.
/// Includes focus ring styles and screen reader helper classes.
#[must_use]
pub const fn visible_focus_css() -> &'static str {
    ":focus{outline:3px solid #00D9C0;outline-offset:2px} .sr-only{position:absolute;width:1px;height:1px;margin:-1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap;}"
}

/// Update the live region status for screen readers
///
/// Updates the text content of the #menu-helper element if present.
/// This provides announcements to assistive technology users.
pub fn set_status(msg: &str) {
    if !cfg!(target_arch = "wasm32") {
        let _ = msg;
        return;
    }
    if let Some(node) = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id("menu-helper"))
    {
        node.set_text_content(Some(msg));
    }
}

/// Toggle high-contrast mode for accessibility
///
/// Adds or removes the 'hc' class from the HTML element and persists the choice.
/// This enables high-contrast styling for users with visual impairments.
pub fn set_high_contrast(enabled: bool) {
    if !cfg!(target_arch = "wasm32") {
        let _ = enabled;
        return;
    }
    let Some(win) = web_sys::window() else {
        return;
    };

    if let Some(html) = win.document().and_then(|doc| doc.document_element()) {
        let _ = if enabled {
            html.class_list().add_1("hc")
        } else {
            html.class_list().remove_1("hc")
        };
    }

    if let Some(storage) = win.local_storage().ok().flatten() {
        let _ = storage.set_item("dystrail.hc", if enabled { "1" } else { "0" });
    }
}

/// Check if high-contrast mode is currently enabled
///
/// Reads the saved preference from localStorage to determine if high-contrast
/// styling should be active. Returns false if no preference is stored.
#[must_use]
pub fn high_contrast_enabled() -> bool {
    if !cfg!(target_arch = "wasm32") {
        return false;
    }
    web_sys::window()
        .and_then(|win| win.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("dystrail.hc").ok().flatten())
        .is_some_and(|v| v == "1")
}

/// Restore focus to the element with the provided ID, if it exists.
///
/// This is used when closing transient UI (dialogs, drawers) to return focus
/// to the triggering control and keep keyboard users oriented.
pub fn restore_focus(prev_id: &str) {
    if !cfg!(target_arch = "wasm32") {
        let _ = prev_id;
        return;
    }
    if let Some(doc) = web_sys::window().and_then(|win| win.document())
        && let Some(el) = doc
            .get_element_by_id(prev_id)
            .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = el.focus();
    }
}

/// Trap focus within a container by cycling focusable elements on Tab/Shift+Tab.
///
/// Attaches a keydown handler to the container; leaks the handler intentionally
/// so it survives for the life of the dialog. No-op outside the browser target.
pub fn trap_focus_in(container_id: &str) {
    if !cfg!(target_arch = "wasm32") {
        let _ = container_id;
        return;
    }
    let Some(doc) = web_sys::window().and_then(|win| win.document()) else {
        return;
    };
    let Some(container) = doc.get_element_by_id(container_id) else {
        return;
    };
    let selector = "a[href], button, textarea, input, select, [tabindex]:not([tabindex=\"-1\"])";
    let Ok(node_list) = container.query_selector_all(selector) else {
        return;
    };

    let mut focusables: Vec<web_sys::HtmlElement> = Vec::new();
    for idx in 0..node_list.length() {
        if let Some(el) = node_list
            .get(idx)
            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
        {
            focusables.push(el);
        }
    }

    if focusables.is_empty() {
        return;
    }

    let closure = Closure::wrap(Box::new(move |evt: web_sys::KeyboardEvent| {
        if evt.key() != "Tab" {
            return;
        }
        let active = doc.active_element();
        let first = focusables.first();
        let last = focusables.last();
        if evt.shift_key() {
            if let (Some(active_el), Some(last_el)) = (active.as_ref(), last)
                && active_el.is_same_node(Some(first.unwrap_or(last_el)))
            {
                evt.prevent_default();
                let _ = last_el.focus();
            }
        } else if let (Some(active_el), Some(first_el)) = (active.as_ref(), first)
            && active_el.is_same_node(Some(last.unwrap_or(first_el)))
        {
            evt.prevent_default();
            let _ = first_el.focus();
        }
    }) as Box<dyn FnMut(_)>);

    let _ = container.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
    closure.forget();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_css_includes_sr_only_helpers() {
        let css = visible_focus_css();
        assert!(css.contains(":focus"));
        assert!(css.contains(".sr-only"));
    }

    #[test]
    fn status_helpers_are_noops_without_dom() {
        // Should not panic when no browser window exists.
        set_status("Testing status update");
        set_high_contrast(true);
        set_high_contrast(false);
        assert!(!high_contrast_enabled());
        restore_focus("missing");
        trap_focus_in("missing");
    }
}
