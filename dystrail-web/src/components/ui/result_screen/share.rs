use crate::dom;
use crate::game::{ResultSummary, result_summary};
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

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

pub(super) fn interpolate_template(
    template: &str,
    summary: &ResultSummary,
    headline_text: &str,
) -> String {
    template
        .replace("{headline}", headline_text)
        .replace(
            "{score}",
            &crate::i18n::fmt_number(f64::from(summary.score)),
        )
        .replace("{seed}", &summary.seed)
        .replace("{persona}", &summary.persona_name)
        .replace("{mult}", &summary.mult_str)
        .replace("{mode}", &summary.mode)
}

pub(super) fn copy_payload(text: &str) -> Result<(), String> {
    fallback_copy(text)
}

fn fallback_copy(text: &str) -> Result<(), String> {
    let Some(document) = dom::document() else {
        return Err("Document unavailable".to_string());
    };
    let textarea = document
        .create_element("textarea")
        .map_err(|_| "Failed to create textarea".to_string())?
        .dyn_into::<HtmlTextAreaElement>()
        .map_err(|_| "Failed to cast to textarea".to_string())?;

    textarea.set_value(text);

    if let Ok(style) = js_sys::Reflect::get(&textarea, &"style".into()) {
        let _ = js_sys::Reflect::set(&style, &"position".into(), &"fixed".into());
        let _ = js_sys::Reflect::set(&style, &"top".into(), &"-1000px".into());
        let _ = js_sys::Reflect::set(&style, &"left".into(), &"-1000px".into());
    }

    if let Some(body) = document.body() {
        body.append_child(&textarea)
            .map_err(|_| "Failed to append textarea".to_string())?;
        textarea.select();
        body.remove_child(&textarea)
            .map_err(|_| "Failed to remove textarea".to_string())?;
        Ok(())
    } else {
        Err("No body element".to_string())
    }
}
