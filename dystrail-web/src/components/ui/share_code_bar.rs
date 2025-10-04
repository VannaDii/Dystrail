use crate::i18n;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    pub value: AttrValue,
    pub valid: bool,
    pub onchange: Callback<String>,
    pub onstart: Callback<()>,
}

#[function_component(ShareCodeBar)]
pub fn share_code_bar(p: &Props) -> Html {
    let oninput = {
        let cb = p.onchange.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                cb.emit(input.value());
            }
        })
    };
    let onstart = {
        let cb = p.onstart.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! {
        <div class="panel">
            <label for="code"><strong>{ i18n::t("share.code") }</strong></label>
            <div class="controls">
                <input
                    id="code"
                    type="text"
                    class="share-code-input"
                    value={p.value.clone()}
                    {oninput}
                    aria-invalid={(!p.valid).then(|| AttrValue::from("true"))}
                    aria-describedby={Some(AttrValue::from("code-help"))}
                    placeholder="CL-ORANGE42"
                />
                <button class="retro-btn-primary" onclick={onstart} disabled={!p.valid}>
                    { i18n::t("share.start_with_code") }
                </button>
            </div>
            <p id="code-help" class="muted">{ i18n::t("share.code_help") }</p>
        </div>
    }
}
