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
fn travel_panel_journal_is_available_during_a_breakdown() {
    crate::i18n::set_lang("en");

    let html = block_on(
        LocalServerRenderer::<TravelPanel>::with_props(Props {
            detail: 1,
            receipt: yew::Html::default(),
            logs: vec!["Welcome back".into()],
            game_state: Some(sample_game_state()),
            pacing_config: Rc::new(PacingConfig::default_config()),
            on_pace_change: Callback::noop(),
            on_diet_change: Callback::noop(),
        })
        .render(),
    );

    assert!(
        html.contains("Trail journal"),
        "Journal access should appear: {html}"
    );
    assert!(
        html.contains("Welcome back"),
        "Journal logs should appear: {html}"
    );
}

#[test]
fn weather_readout_reports_applied_exposure_without_spending_again() {
    crate::i18n::set_lang("en");
    let cfg = WeatherConfig::default_config();
    let mut gs = GameState::default();
    gs.weather_state.today = Weather::HeatWave;
    gs.exposure_streak_heat = 2;
    crate::game::weather::apply_weather_effects(&mut gs, &cfg);
    let before = gs.stats.clone();
    let readout = crate::components::ui::stats_bar::weather::readout(&gs, &cfg);
    assert!(readout.cost.contains("Sanity -2"));
    assert!(readout.cost.contains("Health -1"));
    assert_eq!(readout.applied_day, Some(1));
    assert_eq!(readout.changes.len(), 3);
    assert_eq!(readout.rates.len(), 2);
    let _ = crate::components::ui::stats_bar::weather::details(&readout);
    assert_eq!(gs.stats, before);
    gs.weather_state.today = Weather::ColdSnap;
    gs.inventory.tags.insert("cold_resist".into());
    crate::game::weather::apply_weather_effects(&mut gs, &cfg);
    let protected = crate::components::ui::stats_bar::weather::readout(&gs, &cfg);
    assert_eq!(protected.cost, "Distance -10%");
    gs.weather_state.today = Weather::Clear;
    let clear = crate::components::ui::stats_bar::weather::readout(&gs, &cfg);
    assert_eq!(clear.cost, "No weather cost");
    assert!(!clear.cost.contains("Health"));
}
