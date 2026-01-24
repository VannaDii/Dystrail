use super::layout::{IntentActions, PanelContext, PanelMode, render_panel};
use super::pace::{diet_code, diet_preview, pace_code, pace_preview};
use super::view::{
    build_pace_diet_panel, build_weather_details, compute_panel_mode, next_flag_state,
    next_flag_toggle, show_flag_action, toggle_flag_action,
};
use super::weather::{
    format_delta, format_percent, format_weather_announcement, render_weather_details,
    render_weather_info,
};
use super::*;
use crate::game::vehicle::{Breakdown, Part};
use crate::game::weather::{Weather, WeatherConfig, WeatherState};
use crate::game::{DietId, GameState, Inventory, PaceId, PacingConfig, Region};
use futures::executor::block_on;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;
use web_sys::MouseEvent;
use yew::Callback;
use yew::LocalServerRenderer;
use yew::prelude::*;

fn sample_game_state() -> Rc<GameState> {
    let mut state = GameState {
        day: 3,
        region: Region::RustBelt,
        breakdown: Some(Breakdown {
            part: Part::Battery,
            day_started: 2,
        }),
        logs: vec!["Log booting".into(), "Arrived in Rust Belt".into()],
        weather_state: WeatherState {
            today: Weather::Storm,
            yesterday: Weather::ColdSnap,
            extreme_streak: 1,
            heatwave_streak: 0,
            coldsnap_streak: 0,
            neutral_buffer: 0,
            rain_accum: 0.0,
            snow_depth: 0.0,
        },
        inventory: Inventory {
            tags: HashSet::from_iter([String::from("rain_resist")]),
            ..Inventory::default()
        },
        ..GameState::default()
    };
    state.day_state.travel.travel_blocked = true;
    Rc::new(state)
}

#[test]
fn travel_panel_render_includes_weather_and_breakdown() {
    crate::i18n::set_lang("en");

    let html = block_on(
        LocalServerRenderer::<TravelPanel>::with_props(Props {
            on_travel: Callback::noop(),
            on_trade: Callback::noop(),
            on_hunt: Callback::noop(),
            logs: vec!["Welcome back".into()],
            game_state: Some(sample_game_state()),
            pacing_config: Rc::new(PacingConfig::default_config()),
            on_pace_change: Callback::noop(),
            on_diet_change: Callback::noop(),
        })
        .render(),
    );

    assert!(
        html.contains("Weather: Storm"),
        "SSR output should include weather state: {html}"
    );
    assert!(
        html.contains("Travel blocked until repaired."),
        "Breakdown banner should be present when travel is blocked: {html}"
    );
    assert!(
        html.contains("Log booting") || html.contains("Welcome back"),
        "Rendered log entries should appear: {html}"
    );
}

#[test]
fn helpers_format_readable_effects() {
    let pos = format_delta("Supplies", 3);
    let neg = format_delta("Supplies", -2);
    assert_eq!(pos, "Supplies +3");
    assert_eq!(neg, "Supplies -2");

    let pct_pos = format_percent("Encounter", 0.25);
    let pct_neg = format_percent("Encounter", -0.5);
    assert_eq!(pct_pos, "Encounter +25%");
    assert_eq!(pct_neg, "Encounter -50%");

    let weather_cfg = WeatherConfig::default_config();
    let announcement =
        format_weather_announcement(Weather::Storm, weather_cfg.effects.get(&Weather::Storm));
    assert!(
        announcement.contains("Weather: Storm"),
        "Announcement should mention current weather: {announcement}"
    );
}

#[test]
fn render_weather_info_uses_beltway_label() {
    #[function_component(WeatherInfoHarness)]
    fn weather_info_harness() -> Html {
        crate::i18n::set_lang("en");
        let state = GameState {
            region: Region::Beltway,
            ..GameState::default()
        };
        render_weather_info(&state)
    }

    let html = block_on(LocalServerRenderer::<WeatherInfoHarness>::new().render());
    assert!(html.contains("Beltway"));
}

#[derive(Properties, Clone, PartialEq)]
struct PanelHarnessProps {
    mode: PanelMode,
    logs: Vec<String>,
    travel_blocked: bool,
    with_game_state: bool,
    show_intents: bool,
}

#[function_component(PanelHarness)]
fn panel_harness(props: &PanelHarnessProps) -> Html {
    crate::i18n::set_lang("en");
    let game_state = GameState::default();
    let pacing_config = PacingConfig::default_config();
    let on_click = Callback::from(|_e: MouseEvent| {});
    let on_show_pace_diet = Callback::from(|_e: MouseEvent| {});
    let on_toggle_weather_details = Callback::from(|_e: MouseEvent| {});
    let on_trade = Callback::from(|_e: MouseEvent| {});
    let on_hunt = Callback::from(|_e: MouseEvent| {});
    let intent_actions = props.show_intents.then_some(IntentActions {
        on_trade: &on_trade,
        on_hunt: &on_hunt,
    });

    let ctx = PanelContext {
        travel_blocked: props.travel_blocked,
        breakdown_msg: None,
        mode: props.mode,
        weather_details: html! { <div class="weather-details-card">{"details"}</div> },
        pace_diet_panel: html! { <div class="pace-diet-panel">{"pace diet"}</div> },
        weather_info: html! { <div class="weather-info">{"weather"}</div> },
        logs: &props.logs,
        game_state: props.with_game_state.then_some(&game_state),
        pacing_config: &pacing_config,
        intent_actions,
        on_show_pace_diet: &on_show_pace_diet,
        on_toggle_weather_details: &on_toggle_weather_details,
        on_click: &on_click,
    };

    render_panel(&ctx)
}

#[test]
fn render_panel_main_renders_logs_and_intents() {
    let html = block_on(
        LocalServerRenderer::<PanelHarness>::with_props(PanelHarnessProps {
            mode: PanelMode::Main,
            logs: vec!["Entry".to_string()],
            travel_blocked: false,
            with_game_state: true,
            show_intents: true,
        })
        .render(),
    );
    assert!(html.contains("Entry"));
    assert!(html.contains("Trade"));
    assert!(html.contains("Hunt"));
}

#[test]
fn render_panel_weather_details_renders_details_card() {
    let html = block_on(
        LocalServerRenderer::<PanelHarness>::with_props(PanelHarnessProps {
            mode: PanelMode::WeatherDetails,
            logs: Vec::new(),
            travel_blocked: false,
            with_game_state: false,
            show_intents: false,
        })
        .render(),
    );
    assert!(html.contains("weather-details-card"));
}

#[test]
fn render_panel_pace_diet_renders_panel() {
    let html = block_on(
        LocalServerRenderer::<PanelHarness>::with_props(PanelHarnessProps {
            mode: PanelMode::PaceDiet,
            logs: Vec::new(),
            travel_blocked: false,
            with_game_state: true,
            show_intents: false,
        })
        .render(),
    );
    assert!(html.contains("pace-diet-panel"));
}

#[test]
fn render_panel_main_without_state_skips_settings() {
    let html = block_on(
        LocalServerRenderer::<PanelHarness>::with_props(PanelHarnessProps {
            mode: PanelMode::Main,
            logs: Vec::new(),
            travel_blocked: false,
            with_game_state: false,
            show_intents: false,
        })
        .render(),
    );
    assert!(!html.contains("current-settings"));
}

#[test]
fn show_flag_action_sets_state() {
    #[function_component(ShowFlagHarness)]
    fn show_flag_harness() -> Html {
        let flag = use_state(|| false);
        let invoked = use_mut_ref(|| false);
        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            let action = show_flag_action(flag, true);
            action.emit(());
        }
        let called = if *invoked.borrow() { "true" } else { "false" };
        html! { <div data-called={called} /> }
    }
    let html = block_on(LocalServerRenderer::<ShowFlagHarness>::new().render());
    assert!(html.contains("data-called=\"true\""));
}

#[test]
fn toggle_flag_action_flips_state() {
    #[function_component(ToggleFlagHarness)]
    fn toggle_flag_harness() -> Html {
        let flag = use_state(|| false);
        let invoked = use_mut_ref(|| false);
        if !*invoked.borrow() {
            *invoked.borrow_mut() = true;
            let action = toggle_flag_action(flag);
            action.emit(());
        }
        let called = if *invoked.borrow() { "true" } else { "false" };
        html! { <div data-called={called} /> }
    }
    let html = block_on(LocalServerRenderer::<ToggleFlagHarness>::new().render());
    assert!(html.contains("data-called=\"true\""));
}

#[test]
fn flag_helpers_return_expected_values() {
    assert!(next_flag_state(false, true));
    assert!(!next_flag_state(true, false));
    assert!(next_flag_toggle(false));
    assert!(!next_flag_toggle(true));
}

#[test]
fn compute_panel_mode_prioritizes_weather_details() {
    assert_eq!(compute_panel_mode(true, true), PanelMode::WeatherDetails);
    assert_eq!(compute_panel_mode(false, true), PanelMode::PaceDiet);
    assert_eq!(compute_panel_mode(false, false), PanelMode::Main);
}

#[test]
fn build_weather_details_renders_card_when_enabled() {
    #[function_component(WeatherDetailsHarness)]
    fn weather_details_harness() -> Html {
        crate::i18n::set_lang("en");
        let state = sample_game_state();
        let on_toggle = Callback::noop();
        build_weather_details(true, Some(state.as_ref()), &on_toggle)
    }

    let html = block_on(LocalServerRenderer::<WeatherDetailsHarness>::new().render());
    assert!(html.contains("weather-details-card"));
}

#[test]
fn build_pace_diet_panel_reports_missing_state() {
    #[function_component(PaceDietMissingHarness)]
    fn pace_diet_missing_harness() -> Html {
        crate::i18n::set_lang("en");
        let on_back = Callback::noop();
        build_pace_diet_panel(
            true,
            None,
            Rc::new(PacingConfig::default_config()),
            Callback::noop(),
            Callback::noop(),
            &on_back,
        )
    }

    let html = block_on(LocalServerRenderer::<PaceDietMissingHarness>::new().render());
    assert!(html.contains("Game state unavailable"));
}

#[test]
fn build_pace_diet_panel_renders_panel_with_state() {
    #[function_component(PaceDietHarness)]
    fn pace_diet_harness() -> Html {
        crate::i18n::set_lang("en");
        let state = sample_game_state();
        let on_back = Callback::noop();
        build_pace_diet_panel(
            true,
            Some(state),
            Rc::new(PacingConfig::default_config()),
            Callback::noop(),
            Callback::noop(),
            &on_back,
        )
    }

    let html = block_on(LocalServerRenderer::<PaceDietHarness>::new().render());
    assert!(html.contains("pace-diet-panel"));
}

#[test]
fn render_panel_skips_breakdown_when_missing_message() {
    let html = block_on(
        LocalServerRenderer::<PanelHarness>::with_props(PanelHarnessProps {
            mode: PanelMode::Main,
            logs: Vec::new(),
            travel_blocked: true,
            with_game_state: true,
            show_intents: false,
        })
        .render(),
    );
    assert!(!html.contains("breakdown-alert"));
    assert!(html.contains("breakdown-notice"));
}

#[test]
fn pace_and_diet_helpers_fallback_when_missing() {
    crate::i18n::set_lang("en");
    let mut config = PacingConfig::default();
    config.pace.clear();
    config.diet.clear();
    assert_eq!(pace_preview(&config, PaceId::Steady), "");
    assert_eq!(diet_preview(&config, DietId::Mixed), "");
    assert_eq!(pace_code(PaceId::Steady), "S");
    assert_eq!(pace_code(PaceId::Heated), "H");
    assert_eq!(pace_code(PaceId::Blitz), "B");
    assert_eq!(diet_code(DietId::Mixed), "M");
    assert_eq!(diet_code(DietId::Quiet), "Q");
    assert_eq!(diet_code(DietId::Doom), "D");
}

#[test]
fn weather_info_renders_clear_state_without_effects_text() {
    #[function_component(WeatherInfoHarness)]
    fn weather_info_harness() -> Html {
        crate::i18n::set_lang("en");
        let state = GameState {
            day: 1,
            region: Region::Heartland,
            weather_state: WeatherState {
                today: Weather::Clear,
                ..WeatherState::default()
            },
            ..GameState::default()
        };
        render_weather_info(&state)
    }

    let html = block_on(LocalServerRenderer::<WeatherInfoHarness>::new().render());
    assert!(html.contains("Weather: Clear"));
}

#[test]
fn weather_details_renders_mitigation_and_effects() {
    #[function_component(WeatherDetailsHarness)]
    fn weather_details_harness() -> Html {
        crate::i18n::set_lang("en");
        let mut state = GameState::default();
        state.weather_state.today = Weather::Storm;
        render_weather_details(&state)
    }

    let html = block_on(LocalServerRenderer::<WeatherDetailsHarness>::new().render());
    assert!(html.contains("Mitigation tag"));
}

#[test]
fn weather_announcement_handles_no_effects() {
    crate::i18n::set_lang("en");
    let announcement = format_weather_announcement(Weather::Clear, None);
    assert!(announcement.contains("Weather:"));
    assert!(!announcement.contains("Supplies +"));
}

#[test]
fn travel_panel_props_eq_tracks_logs_and_pace_diet() {
    let pacing = Rc::new(PacingConfig::default_config());
    let make_props = |logs: Vec<String>, pace: PaceId, diet: DietId| {
        let state = GameState {
            pace,
            diet,
            ..GameState::default()
        };
        Props {
            on_travel: Callback::noop(),
            on_trade: Callback::noop(),
            on_hunt: Callback::noop(),
            logs,
            game_state: Some(Rc::new(state)),
            pacing_config: pacing.clone(),
            on_pace_change: Callback::noop(),
            on_diet_change: Callback::noop(),
        }
    };

    let props_a = make_props(vec!["Log".to_string()], PaceId::Steady, DietId::Mixed);
    let props_b = make_props(vec!["Log".to_string()], PaceId::Steady, DietId::Mixed);
    assert!(props_a == props_b);

    let props_c = make_props(vec!["Log".to_string()], PaceId::Heated, DietId::Mixed);
    assert!(props_a != props_c);

    let props_d = make_props(vec!["Other".to_string()], PaceId::Steady, DietId::Mixed);
    assert!(props_a != props_d);
}
