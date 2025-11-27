use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub open: bool,
    pub on_close: Callback<()>,
    #[prop_or_default]
    pub on_hc_changed: Callback<bool>,
}

#[function_component(SettingsDialog)]
pub fn settings_dialog(p: &Props) -> Html {
    let ref_node = use_node_ref();
    let close = {
        let cb = p.on_close.clone();
        Callback::from(move |_| cb.emit(()))
    };
    let hc = use_state(crate::a11y::high_contrast_enabled);

    {
        let node = ref_node.clone();
        let open = p.open;
        use_effect_with((open, node), move |(open, node)| {
            let mut prev_focus: Option<web_sys::HtmlElement> = None;
            let focus_target = if cfg!(target_arch = "wasm32") && *open {
                prev_focus = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|doc| {
                        doc.active_element()
                            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
                    });

                node.cast::<web_sys::Element>().and_then(|el| {
                    el.query_selector_all(
                        "button, [href], input, textarea, select, [tabindex]:not([tabindex='-1'])",
                    )
                    .ok()
                    .and_then(|list| {
                        list.get(0)
                            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
                    })
                })
            } else {
                None
            };

            if let Some(first) = focus_target {
                let _ = first.focus();
            }
            move || {
                if let Some(el) = prev_focus {
                    let _ = el.focus();
                }
            }
        });
    }

    if !p.open {
        return html! {};
    }

    let on_keydown = {
        let node = ref_node.clone();
        let on_close = p.on_close.clone();
        Callback::from(move |e: KeyboardEvent| {
            if !cfg!(target_arch = "wasm32") {
                let _ = e;
                return;
            }
            if e.key() == "Escape" {
                on_close.emit(());
                return;
            }
            if e.key() != "Tab" {
                return;
            }
            let Some(container) = node.cast::<web_sys::Element>() else {
                return;
            };
            let Ok(list) = container.query_selector_all(
                "button, [href], input, textarea, select, [tabindex]:not([tabindex='-1'])",
            ) else {
                return;
            };
            let len = list.length();
            if len == 0 {
                return;
            }
            let first = list
                .get(0)
                .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
            let last = list
                .get(len - 1)
                .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
            let active = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element());
            let shift = e.shift_key();
            if let (Some(first), Some(last), Some(active)) = (first, last, active) {
                let first_el: web_sys::Element = first.clone().unchecked_into();
                let last_el: web_sys::Element = last.clone().unchecked_into();
                let is_first = active == first_el;
                let is_last = active == last_el;
                if !container.contains(Some(&active)) {
                    e.prevent_default();
                    let _ = first.focus();
                    return;
                }
                if shift && is_first {
                    e.prevent_default();
                    let _ = last.focus();
                } else if !shift && is_last {
                    e.prevent_default();
                    let _ = first.focus();
                }
            }
        })
    };

    let on_toggle_hc = {
        let hc = hc.clone();
        let hc_cb = p.on_hc_changed.clone();
        Callback::from(move |_| {
            let next = !*hc;
            hc.set(next);
            crate::a11y::set_high_contrast(next);
            hc_cb.emit(next);
        })
    };

    html! {
      <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="settings-title" ref={ref_node} onkeydown={on_keydown}>
        <div class="drawer-body">
          <h2 id="settings-title">{"Settings"}</h2>
          <div class="field">
            <label for="hc-toggle"><strong>{"High Contrast"}</strong></label>
            <input id="hc-toggle" type="checkbox" checked={*hc} onclick={on_toggle_hc} />
          </div>
          <div class="controls">
            <button onclick={close}>{ crate::i18n::t("dialogs.close") }</button>
          </div>
        </div>
      </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn settings_dialog_returns_empty_when_closed() {
        crate::i18n::set_lang("en");
        let props = Props {
            open: false,
            on_close: Callback::noop(),
            on_hc_changed: Callback::noop(),
        };
        let html = block_on(LocalServerRenderer::<SettingsDialog>::with_props(props).render());
        assert!(!html.contains("drawer-body"));
    }

    #[test]
    fn settings_dialog_renders_controls_when_open() {
        crate::i18n::set_lang("en");
        let props = Props {
            open: true,
            on_close: Callback::noop(),
            on_hc_changed: Callback::noop(),
        };
        let html = block_on(LocalServerRenderer::<SettingsDialog>::with_props(props).render());
        assert!(html.contains("High Contrast"));
    }
}
