use super::{Props, SettingsDialog, view::toggle_high_contrast};
use futures::executor::block_on;
use yew::Callback;
use yew::LocalServerRenderer;
use yew::prelude::*;

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

#[test]
fn toggle_high_contrast_updates_state() {
    #[function_component(HcHarness)]
    fn hc_harness() -> Html {
        let hc = use_state(|| false);
        let called = use_mut_ref(|| false);
        let observed = use_mut_ref(|| String::from("false"));
        let on_change = {
            let observed = observed.clone();
            Callback::from(move |next: bool| {
                *observed.borrow_mut() = next.to_string();
            })
        };
        if !*called.borrow() {
            *called.borrow_mut() = true;
            toggle_high_contrast(&hc, &on_change);
        }
        let observed_value = observed.borrow().clone();
        html! { <div data-hc={observed_value} data-state={hc.to_string()} /> }
    }

    let html = block_on(LocalServerRenderer::<HcHarness>::new().render());
    assert!(html.contains("data-hc=\"true\""));
}
