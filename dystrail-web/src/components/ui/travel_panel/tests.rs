use super::weather::{format_delta, format_percent, format_weather_announcement};
use super::*;
use crate::game::vehicle::{Breakdown, Part};
use crate::game::weather::{Weather, WeatherConfig, WeatherState};
use crate::game::{GameState, Inventory, PacingConfig, Region};
use futures::executor::block_on;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;
use yew::Callback;
use yew::LocalServerRenderer;

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
