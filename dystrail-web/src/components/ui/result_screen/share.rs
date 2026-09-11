use crate::dom;
use crate::game::{ResultSummary, result_summary};

pub(super) fn summary(props: &super::Props) -> Result<ResultSummary, String> {
    result_summary(&props.game_state, &props.result_config)
}

pub(super) fn resolved_headline_key(summary: &ResultSummary, props: &super::Props) -> String {
    if props.game_state.continuity.abandoned {
        return "journey.abandoned".into();
    }
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
    if props.game_state.continuity.abandoned {
        return "journey.abandoned_story".into();
    }
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

pub(super) fn copy_payload(text: &str) -> Result<js_sys::Promise, String> {
    let window = dom::window().ok_or_else(|| String::from("Window unavailable"))?;
    Ok(window.navigator().clipboard().write_text(text))
}
