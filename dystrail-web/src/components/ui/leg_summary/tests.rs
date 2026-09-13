use super::*;
use crate::game::{Stats, journal::ResourceChange};

fn entry(day: u32, kind: &str, before: i32, after: i32) -> JournalEntry {
    JournalEntry {
        action_kind: kind.into(),
        day,
        minute: crate::game::travel_time::TRAVEL_DAY_START + 60,
        pace: None,
        diet: None,
        place: String::new(),
        title: "سافرت".into(),
        message: String::new(),
        before: Stats {
            supplies: before,
            ..Stats::default()
        },
        after: Stats {
            supplies: after,
            ..Stats::default()
        },
        details: Vec::new(),
        resources: Vec::new(),
    }
}

fn state(day: u32, supplies: i32, entries: Vec<JournalEntry>) -> GameState {
    let mut gs = GameState {
        day,
        stats: Stats {
            supplies,
            ..Stats::default()
        },
        ..GameState::default()
    };
    gs.continuity.journal = entries;
    gs
}

#[test]
fn unfinished_days_and_non_travel_days_do_not_establish_an_outlook() {
    let gs = state(
        2,
        14,
        vec![entry(1, "camp", 20, 18), entry(2, "travel", 18, 14)],
    );
    assert_eq!(supply_outlook(&gs), SupplyOutlook::Waiting);
}

#[test]
fn localized_titles_and_repeated_hourly_snapshots_preserve_daily_net_use() {
    let gs = state(
        2,
        17,
        vec![
            entry(1, "travel", 20, 18),
            entry(1, "travel", 20, 18),
            entry(1, "travel", 18, 17),
            entry(1, "travel", 17, 17),
        ],
    );
    let before = serde_json::to_string(&gs.continuity).unwrap();
    assert_eq!(supply_outlook(&gs), SupplyOutlook::Days(5.0));
    assert_eq!(serde_json::to_string(&gs.continuity).unwrap(), before);
}

#[test]
fn rollover_costs_close_the_original_day_in_old_and_new_saves() {
    let mut final_hour = entry(2, "travel", 18, 16);
    final_hour.minute = crate::game::travel_time::TRAVEL_DAY_START;
    final_hour.resources.push(ResourceChange {
        key: "play.elapsed".into(),
        before: 1,
        after: 2,
    });
    let mut gs = state(
        2,
        15,
        vec![
            entry(1, "travel", 20, 18),
            final_hour,
            entry(2, "travel", 16, 15),
        ],
    );
    assert_eq!(supply_outlook(&gs), SupplyOutlook::Days(3.0));
    // Corrected journal timestamps still refer to the same completed day.
    gs.continuity.journal[1].day = 1;
    assert_eq!(supply_outlook(&gs), SupplyOutlook::Days(3.0));
}

#[test]
fn purchases_and_care_change_the_days_actual_net_use() {
    let gs = state(
        2,
        16,
        vec![
            entry(1, "travel", 20, 16),
            entry(1, "town", 16, 20),
            entry(1, "care", 20, 18),
            entry(1, "travel", 18, 16),
        ],
    );
    assert_eq!(supply_outlook(&gs), SupplyOutlook::Days(4.0));
}

#[test]
fn steady_stock_days_count_without_cancelling_other_days_consumption() {
    let gs = state(
        3,
        20,
        vec![
            entry(1, "travel", 20, 18),
            entry(2, "town", 18, 20),
            entry(2, "travel", 20, 20),
        ],
    );
    assert_eq!(supply_outlook(&gs), SupplyOutlook::Days(20.0));
    let stable = state(2, 20, vec![entry(1, "travel", 20, 20)]);
    assert_eq!(supply_outlook(&stable), SupplyOutlook::Stable);
    let empty = state(2, 0, vec![entry(1, "travel", 2, 0)]);
    assert_eq!(supply_outlook(&empty), SupplyOutlook::Days(0.0));
    let still_empty = state(2, 0, vec![entry(1, "travel", 0, 0)]);
    assert_eq!(supply_outlook(&still_empty), SupplyOutlook::Days(0.0));
}

#[test]
fn outlook_uses_eight_completed_travel_days_instead_of_eight_actions() {
    let mut entries = vec![entry(1, "travel", 20, 16)];
    let mut supplies = 16;
    for day in 2..=9 {
        entries.push(entry(day, "travel", supplies, supplies - 1));
        supplies -= 1;
        for _ in 0..4 {
            entries.push(entry(day, "travel", supplies, supplies));
        }
    }
    let gs = state(10, supplies, entries);
    assert_eq!(supply_outlook(&gs), SupplyOutlook::Days(8.0));
}

#[function_component(OutlookPreview)]
fn outlook_preview() -> Html {
    outlook(&state(2, 16, vec![entry(1, "travel", 20, 16)]))
}

#[test]
fn outlook_renders_a_shared_stat_card_with_context() {
    i18n::set_lang("en");
    let html =
        futures::executor::block_on(yew::LocalServerRenderer::<OutlookPreview>::new().render());
    assert!(html.contains("impact-details supply-outlook"));
    assert!(html.contains("data-stat=\"journey.forecast_days\""));
    assert!(html.contains("<strong><bdi>4</bdi></strong>"));
    assert!(html.contains(&i18n::t("journey.forecast_basis")));
    assert!(html.contains(&i18n::t("journey.forecast_caveat")));
}
