use super::{Props, SaveDrawer};
use futures::executor::block_on;
use yew::Callback;
use yew::LocalServerRenderer;

fn base_props(open: bool) -> Props {
    Props {
        open,
        on_close: Callback::noop(),
        on_save: Callback::noop(),
        on_load: Callback::noop(),
        on_export: Callback::noop(),
        on_import: Callback::from(|_s: String| {}),
        return_focus_id: None,
    }
}

#[test]
fn save_drawer_hidden_when_closed() {
    crate::i18n::set_lang("en");
    let html = block_on(LocalServerRenderer::<SaveDrawer>::with_props(base_props(false)).render());
    assert!(!html.contains("drawer-body"));
}

#[test]
fn save_drawer_renders_action_buttons_when_open() {
    crate::i18n::set_lang("en");
    let html = block_on(LocalServerRenderer::<SaveDrawer>::with_props(base_props(true)).render());
    assert!(html.contains("drawer-body"));
    assert!(
        html.contains("textarea"),
        "import textarea should be present: {html}"
    );
}
