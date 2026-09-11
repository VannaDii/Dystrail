//! Accessible journey setup: mode is explicit and encoded in the optional replay code.
use crate::components::ui::journey_scene::SceneStage;
use crate::i18n;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct BootPageProps {
    pub logo_src: AttrValue,
    pub ready: bool,
    pub preload_progress: u8,
    pub on_begin: Callback<()>,
    pub code: AttrValue,
    pub on_code_change: Callback<AttrValue>,
}

#[function_component(BootPage)]
pub fn boot_page(p: &BootPageProps) -> Html {
    let deep = p.code.starts_with("DP-");
    let valid = crate::game::seed::decode_to_seed(&p.code).is_some();
    let on_input = {
        let cb = p.on_code_change.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                cb.emit(input.value().trim().to_ascii_uppercase().into());
            }
        })
    };
    let mode_change = |is_deep: bool| {
        let cb = p.on_code_change.clone();
        Callback::from(move |_| {
            let entropy = js_sys::Date::now().to_bits();
            cb.emit(crate::game::seed::generate_code_from_entropy(is_deep, entropy).into());
        })
    };
    let start = {
        let cb = p.on_begin.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! { <section class="setup-screen" aria-labelledby="screen-title">
        <div class="setup-hero">
            <crate::components::ui::journey_scene::JourneyScene {deep} stage={SceneStage::Setup} />
            <div class="setup-heading"><p class="eyebrow">{i18n::t("ux.tagline")}</p>
            <h1 id="screen-title" tabindex="-1">{"DYSTOPIAN TRAIL"}</h1>
            <p>{i18n::t("ux.premise")}</p></div>
        </div>
        <div class="setup-body">
            <fieldset class="mode-picker"><legend>{i18n::t("ux.mode")}</legend>
                <label class={classes!("mode-option", (!deep).then_some("selected"))}>
                    <input type="radio" name="mode" checked={!deep} onchange={mode_change(false)} />
                    <span><strong>{i18n::t("mode.classic")}</strong><span>{i18n::t("ux.classic_desc")}</span></span>
                </label>
                <label class={classes!("mode-option", deep.then_some("selected"))}>
                    <input type="radio" name="mode" checked={deep} onchange={mode_change(true)} />
                    <span><strong>{i18n::t("mode.deep")}</strong><span>{i18n::t("ux.deep_desc")}</span></span>
                </label>
            </fieldset>
            <div class="seed-entry">
                <label for="run-code">{i18n::t("ux.code")}</label>
                <input id="run-code" dir="ltr" value={p.code.clone()} oninput={on_input} aria-describedby="code-help code-error" aria-invalid={(!valid).to_string()} />
                <p id="code-help" class="muted">{i18n::t("play.seed_hint")}</p>
                <p id="code-error" role="status">{if valid {String::new()} else {i18n::t("ux.code_error")}}</p>
            </div>
            <button class="retro-btn-primary" disabled={!p.ready || !valid} onclick={start}>{i18n::t("ux.begin")}</button>
            if !p.ready {<p role="status">{i18n::t("ux.loading")}</p>}
        </div>
    </section> }
}
