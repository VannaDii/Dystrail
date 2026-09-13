use super::{Props, ResultScreenWrapper, menu, share};
use dystrail_game::{Ending, GameState, ResultConfig, ResultSummary};
use futures::executor::block_on;
use std::rc::Rc;
use yew::{Callback, Html, LocalServerRenderer, Properties, function_component};

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
        vehicle_breakdowns: 1,
        miles_traveled: 1945.0,
        malnutrition_days: 0,
    }
}

fn baseline_props() -> Props {
    Props {
        language: "en".into(),
        game_state: Rc::new(GameState::default()),
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
    Rc::make_mut(&mut props.game_state).boss.outcome.attempted = true;
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
    Rc::make_mut(&mut props.game_state).boss.outcome.attempted = true;
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
    assert!(html.contains("scorecard-title"));
    assert!(!html.contains("<details"));
}

#[derive(Clone, PartialEq, Properties)]
struct MarkupProps {
    content: Html,
}

#[function_component(ResultMarkup)]
fn result_markup(props: &MarkupProps) -> Html {
    props.content.clone()
}

#[test]
fn final_stat_cards_preserve_score_totals_physical_miles_and_threshold_result() {
    crate::i18n::set_lang("en");
    let mut props = baseline_props();
    let gs = Rc::make_mut(&mut props.game_state);
    gs.trail_distance = dystrail_game::route::for_state(gs).unwrap().total_miles;
    gs.miles_traveled_actual = 1523.75;
    for passed in [true, false] {
        let mut summary = baseline_summary();
        summary.passed_threshold = passed;
        let content = super::layout::render_body(
            &props,
            &summary,
            1,
            String::new(),
            Callback::noop(),
            Callback::noop(),
            &Callback::noop(),
        );
        let html = block_on(
            LocalServerRenderer::<ResultMarkup>::with_props(MarkupProps { content }).render(),
        );
        assert_eq!(html.matches("data-stat=\"result.labels.").count(), 12);
        let badge = crate::i18n::t(if passed {
            "result.badges.success"
        } else {
            "result.badges.fail"
        });
        for (key, value) in [
            ("score", "12345"),
            ("days", "42"),
            ("encounters", "12"),
            ("receipts", "3"),
            ("allies", "5"),
            ("supplies", "8"),
            ("credibility", "7"),
            ("breakdowns", "1"),
            ("miles", "1524"),
            ("score_threshold", "10000"),
            ("malnutrition", "0"),
            ("passed_threshold", badge.as_str()),
        ] {
            let card = html
                .split(&format!("data-stat=\"result.labels.{key}\""))
                .nth(1)
                .unwrap()
                .split("</li>")
                .next()
                .unwrap();
            assert!(card.contains(&format!("<strong><bdi>{value}</bdi></strong>")));
            assert!(card.contains(&crate::i18n::t(&format!("result.labels.{key}"))));
        }
        for action in [
            "share-open",
            "action-2",
            "action-3",
            "action-4",
            "action-5",
            "action-0",
        ] {
            assert!(html.contains(&format!("id=\"result-{action}\"")));
        }
    }
}

#[test]
fn explicit_journey_cause_wins_over_readiness_and_score() {
    let mut props = baseline_props();
    for ending in [
        Ending::Collapse {
            cause: dystrail_game::CollapseCause::Disease,
        },
        Ending::SanityLoss,
        Ending::Collapse {
            cause: dystrail_game::CollapseCause::Crossing,
        },
    ] {
        let gs = Rc::make_mut(&mut props.game_state);
        gs.ending = Some(ending);
        gs.boss.readiness.ready = true;
        let summary = share::summary(&props).unwrap();
        assert_eq!(
            share::resolved_headline_key(&summary, &props),
            summary.headline_key
        );
        assert_eq!(
            share::resolved_epilogue_key(&summary, &props),
            summary.epilogue_key
        );
    }
    let gs = Rc::make_mut(&mut props.game_state);
    gs.ending = None;
    let summary = baseline_summary();
    assert_eq!(
        share::resolved_headline_key(&summary, &props),
        "result.headline.incomplete"
    );
    assert_eq!(
        share::resolved_epilogue_key(&summary, &props),
        "result.epilogue.incomplete"
    );
}
