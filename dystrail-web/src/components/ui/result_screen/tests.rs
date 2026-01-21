use super::{Props, ResultScreenWrapper, menu, share};
use dystrail_game::{Ending, GameState, MechanicalPolicyId, ResultConfig, ResultSummary};
use futures::executor::block_on;
use yew::Callback;
use yew::LocalServerRenderer;

fn baseline_summary() -> ResultSummary {
    ResultSummary {
        ending: Ending::BossVictory,
        headline_key: "result.headline.victory".into(),
        epilogue_key: "result.epilogue.victory".into(),
        ending_cause: None,
        seed: "CL-TEST90".into(),
        persona_name: "Organizer".into(),
        mult_str: "1.00×".into(),
        mode: "Classic".into(),
        dp_badge: false,
        score: 12_345,
        score_threshold: 10_000,
        passed_threshold: true,
        days: 42,
        encounters: 12,
        receipts: 3,
        allies: 5,
        supplies: 8,
        credibility: 7,
        pants_pct: 55,
        vehicle_breakdowns: 1,
        miles_traveled: 1945.0,
        malnutrition_days: 0,
    }
}

fn baseline_props() -> Props {
    Props {
        game_state: GameState::default(),
        result_config: ResultConfig::default(),
        boss_won: false,
        on_replay_seed: Callback::noop(),
        on_new_run: Callback::noop(),
        on_title: Callback::noop(),
        on_export: Callback::noop(),
    }
}

#[test]
fn headline_resolution_prefers_boss_flags() {
    let summary = baseline_summary();
    let mut props = baseline_props();
    props.game_state.boss.outcome.attempted = true;
    props.boss_won = false;
    let key = share::resolved_headline_key(&summary, &props);
    assert_eq!(key, "result.headline.boss_loss");

    props.boss_won = true;
    let key = share::resolved_headline_key(&summary, &props);
    assert_eq!(key, "result.headline.victory");
}

#[test]
fn epilogue_resolution_tracks_victory_state() {
    let summary = baseline_summary();
    let mut props = baseline_props();
    props.game_state.boss.readiness.ready = true;
    props.boss_won = false;
    let key = share::resolved_epilogue_key(&summary, &props);
    assert_eq!(key, "result.epilogue.boss_loss");

    props.boss_won = true;
    let key = share::resolved_epilogue_key(&summary, &props);
    assert_eq!(key, "result.epilogue.victory");
}

#[test]
fn parse_numeric_key_identifies_digits() {
    assert_eq!(menu::parse_numeric_key("3"), Some(3));
    assert_eq!(menu::parse_numeric_key("0"), Some(0));
    assert_eq!(menu::parse_numeric_key("A"), None);
}

#[test]
fn result_screen_renders_summary() {
    crate::i18n::set_lang("en");
    let props = baseline_props();
    let html = block_on(LocalServerRenderer::<ResultScreenWrapper>::with_props(props).render());
    assert!(html.contains("result-screen"));
    assert!(html.contains("Result"));
}

#[test]
fn result_screen_hides_thresholds_for_otdeluxe() {
    crate::i18n::set_lang("en");
    let mut props = baseline_props();
    props.game_state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
    let html = block_on(LocalServerRenderer::<ResultScreenWrapper>::with_props(props).render());
    assert!(!html.contains("Score Threshold"));
    assert!(!html.contains("Boss Vote Ready"));
}

#[test]
fn interpolate_template_includes_summary_fields() {
    crate::i18n::set_lang("en");
    let summary = baseline_summary();
    let text = share::interpolate_template(
        "Seed {seed} Score {score} {headline} {persona} {mult} {mode}",
        &summary,
        "Heading",
    );
    assert!(text.contains(&summary.seed));
    assert!(text.contains(&summary.persona_name));
    assert!(text.contains(&summary.mode));
    assert!(text.contains("Heading"));
}

#[test]
fn copy_payload_errors_without_document() {
    let err = share::copy_payload("payload").unwrap_err();
    assert!(err.contains("Document"));
}

#[test]
fn summary_uses_result_config() {
    let props = baseline_props();
    let summary = share::summary(&props).expect("summary should build");
    assert!(!summary.seed.is_empty());
    assert!(!summary.mode.is_empty());
}

#[test]
fn props_eq_tracks_result_config_and_boss_only() {
    let mut props_a = baseline_props();
    let mut props_b = baseline_props();
    props_b.game_state.day = props_a.game_state.day + 5;
    assert!(props_a == props_b);

    props_b.boss_won = true;
    assert!(props_a != props_b);

    props_a.boss_won = true;
    props_b = baseline_props();
    props_b.boss_won = true;
    props_b.result_config.limits.share_seed_maxlen = 99;
    assert!(props_a != props_b);
}
