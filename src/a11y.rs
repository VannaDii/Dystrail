// Accessibility helpers
// Visible focus ring (critical CSS injected at startup)
#[must_use]
pub fn visible_focus_css() -> &'static str {
    ":focus{outline:3px solid #00D9C0;outline-offset:2px} .sr-only{position:absolute;width:1px;height:1px;margin:-1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap;}"
}

// Live region status helper (updates #menu-helper aria-live region if present)
pub fn set_status(msg: &str) {
    if let Some(win) = web_sys::window()
        && let Some(doc) = win.document()
        && let Some(node) = doc.get_element_by_id("menu-helper") {
        node.set_text_content(Some(msg));
    }
}

// High-contrast mode toggle: adds/removes `hc` class on <html> and persists to localStorage
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

#[must_use]
pub fn high_contrast_enabled() -> bool {
    if let Some(win) = web_sys::window()
        && let Ok(Some(storage)) = win.local_storage()
        && let Ok(Some(v)) = storage.get_item("dystrail.hc") {
        return v == "1";
    }
    false
}
