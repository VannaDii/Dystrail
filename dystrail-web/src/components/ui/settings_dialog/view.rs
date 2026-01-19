use super::focus::{keydown_handler, use_focus_management};
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

    use_focus_management(p.open, ref_node.clone());

    if !p.open {
        return html! {};
    }

    let on_keydown = keydown_handler(&ref_node, p.on_close.clone());

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
          <h2 id="settings-title">{ crate::i18n::t("settings.title") }</h2>
          <div class="field">
            <label for="hc-toggle"><strong>{ crate::i18n::t("ui.hc_toggle") }</strong></label>
            <input id="hc-toggle" type="checkbox" checked={*hc} onclick={on_toggle_hc} />
          </div>
          <div class="controls">
            <button onclick={close}>{ crate::i18n::t("dialogs.close") }</button>
          </div>
        </div>
      </div>
    }
}
