use super::Props;
use super::SettingsDialog;
use futures::executor::block_on;
use yew::Callback;
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
    assert!(html.contains("High contrast"));
}
