// Accessibility helpers

/// Get CSS for visible focus indicators and screen reader utilities
///
/// Returns critical accessibility CSS that should be injected early in the page load.
/// Includes focus ring styles and screen reader helper classes.
#[must_use]
pub fn visible_focus_css() -> &'static str {
    ":focus{outline:3px solid #00D9C0;outline-offset:2px} .sr-only{position:absolute;width:1px;height:1px;margin:-1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap;}"
}

/// Update the live region status for screen readers
///
/// Updates the text content of the #menu-helper element if present.
/// This provides announcements to assistive technology users.
pub fn set_status(msg: &str) {
    if let Some(win) = web_sys::window()
        && let Some(doc) = win.document()
        && let Some(node) = doc.get_element_by_id("menu-helper") {
        node.set_text_content(Some(msg));
    }
}

/// Toggle high-contrast mode for accessibility
///
/// Adds or removes the 'hc' class from the HTML element and persists the choice.
/// This enables high-contrast styling for users with visual impairments.
pub fn set_high_contrast(enabled: bool) {
    if let Some(win) = web_sys::window() {
        if let Some(doc) = win.document()
            && let Some(html) = doc.document_element() {
            let _ = if enabled {
                html.class_list().add_1("hc")
            } else {
                html.class_list().remove_1("hc")
            };
        }
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item("dystrail.hc", if enabled { "1" } else { "0" });
        }
    }
}

/// Check if high-contrast mode is currently enabled
///
/// Reads the saved preference from localStorage to determine if high-contrast
/// styling should be active. Returns false if no preference is stored.
#[must_use]
pub fn high_contrast_enabled() -> bool {
    if let Some(win) = web_sys::window()
        && let Ok(Some(storage)) = win.local_storage()
        && let Ok(Some(v)) = storage.get_item("dystrail.hc") {
        v == "1"
    } else {
        false
    }
}
