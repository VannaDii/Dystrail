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
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                    cb.emit(input.value());
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (&e, &cb);
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;

    #[test]
    fn share_code_bar_marks_invalid_codes() {
        crate::i18n::set_lang("en");
        let props = Props {
            value: AttrValue::from("bad-code"),
            valid: false,
            onchange: Callback::from(|_v: String| {}),
            onstart: Callback::noop(),
        };

        let html = block_on(LocalServerRenderer::<ShareCodeBar>::with_props(props).render());
        assert!(
            html.contains("aria-invalid=\"true\""),
            "input should be marked invalid: {html}"
        );
        assert!(
            html.contains("disabled"),
            "start button should be disabled when code invalid: {html}"
        );
    }

    #[test]
    fn share_code_bar_enables_start_for_valid_codes() {
        crate::i18n::set_lang("en");
        let props = Props {
            value: AttrValue::from("CL-ORANGE42"),
            valid: true,
            onchange: Callback::from(|_v: String| {}),
            onstart: Callback::noop(),
        };

        let html = block_on(LocalServerRenderer::<ShareCodeBar>::with_props(props).render());
        assert!(html.contains("CL-ORANGE42"));
        assert!(
            !html.contains("disabled"),
            "start button should not be disabled when code is valid: {html}"
        );
    }
}
