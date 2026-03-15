#[cfg(target_arch = "wasm32")]
use crate::dom;
use crate::game::{ResultSummary, result_summary};
use crate::i18n;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlTextAreaElement;

#[cfg(any(target_arch = "wasm32", test))]
const HEADLINE_PLACEHOLDER: &str = "{headline}";
#[cfg(any(target_arch = "wasm32", test))]
const SCORE_PLACEHOLDER: &str = "{score}";
#[cfg(any(target_arch = "wasm32", test))]
const SEED_PLACEHOLDER: &str = "{seed}";
#[cfg(any(target_arch = "wasm32", test))]
const PERSONA_PLACEHOLDER: &str = "{persona}";
#[cfg(any(target_arch = "wasm32", test))]
const MULT_PLACEHOLDER: &str = "{mult}";
#[cfg(any(target_arch = "wasm32", test))]
const MODE_PLACEHOLDER: &str = "{mode}";

pub(super) fn summary(props: &super::Props) -> Result<ResultSummary, String> {
    result_summary(&props.game_state, &props.result_config)
}

pub(super) fn resolved_headline_key(summary: &ResultSummary, props: &super::Props) -> String {
    if (props.game_state.boss.outcome.attempted || props.game_state.boss.readiness.ready)
        && !props.boss_won
    {
        "result.headline.boss_loss".to_string()
    } else if props.boss_won {
        "result.headline.victory".to_string()
    } else {
        summary.headline_key.clone()
    }
}

pub(super) fn resolved_epilogue_key(summary: &ResultSummary, props: &super::Props) -> String {
    if (props.game_state.boss.outcome.attempted || props.game_state.boss.readiness.ready)
        && !props.boss_won
    {
        "result.epilogue.boss_loss".to_string()
    } else if props.boss_won {
        "result.epilogue.victory".to_string()
    } else {
        summary.epilogue_key.clone()
    }
}

pub(super) fn resolved_persona_name(summary: &ResultSummary) -> String {
    summary
        .persona_key
        .as_deref()
        .map(i18n::t)
        .or_else(|| summary.persona_name.clone())
        .unwrap_or_else(|| i18n::t("persona.traveler.name"))
}

pub(super) fn resolved_mode_name(summary: &ResultSummary) -> String {
    i18n::t(&summary.mode_key)
}

#[cfg(any(target_arch = "wasm32", test))]
pub(super) fn interpolate_template(
    template: &str,
    summary: &ResultSummary,
    headline_text: &str,
) -> String {
    let score = crate::i18n::fmt_number(f64::from(summary.score));
    let persona_name = resolved_persona_name(summary);
    let mode_name = resolved_mode_name(summary);
    let text = template.replace(HEADLINE_PLACEHOLDER, headline_text);
    let text = text.replace(SCORE_PLACEHOLDER, &score);
    let text = text.replace(SEED_PLACEHOLDER, &summary.seed);
    let text = text.replace(PERSONA_PLACEHOLDER, &persona_name);
    let text = text.replace(MULT_PLACEHOLDER, &summary.mult_str);
    text.replace(MODE_PLACEHOLDER, &mode_name)
}

#[cfg(any(target_arch = "wasm32", test))]
pub(super) fn copy_payload(text: &str) -> Result<(), String> {
    fallback_copy(text)
}

#[cfg(all(target_arch = "wasm32", any(target_arch = "wasm32", test)))]
fn fallback_copy(text: &str) -> Result<(), String> {
    let Some(document) = dom::document() else {
        return Err(i18n::t("result.share.errors.document_unavailable"));
    };
    let textarea = document
        .create_element("textarea")
        .map_err(|_| i18n::t("result.share.errors.create_textarea"))?
        .dyn_into::<HtmlTextAreaElement>()
        .map_err(|_| i18n::t("result.share.errors.cast_textarea"))?;

    textarea.set_value(text);

    if let Ok(style) = js_sys::Reflect::get(&textarea, &"style".into()) {
        let _ = js_sys::Reflect::set(&style, &"position".into(), &"fixed".into());
        let _ = js_sys::Reflect::set(&style, &"top".into(), &"-1000px".into());
        let _ = js_sys::Reflect::set(&style, &"left".into(), &"-1000px".into());
    }

    if let Some(body) = document.body() {
        body.append_child(&textarea)
            .map_err(|_| i18n::t("result.share.errors.append_textarea"))?;
        textarea.select();
        body.remove_child(&textarea)
            .map_err(|_| i18n::t("result.share.errors.remove_textarea"))?;
        Ok(())
    } else {
        Err(i18n::t("result.share.errors.no_body"))
    }
}

#[cfg(all(not(target_arch = "wasm32"), test))]
fn fallback_copy(text: &str) -> Result<(), String> {
    let _ = text;
    Err(i18n::t("result.share.errors.document_unavailable"))
}
