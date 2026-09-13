use super::*;
use crate::game::journal::{JournalEntry, ResourceChange};
use crate::game::{DietId, PaceId, Stats};
use futures::executor::block_on;
use yew::LocalServerRenderer;

fn change(key: &str, before: i64, after: i64) -> ResourceChange {
    ResourceChange {
        key: key.into(),
        before,
        after,
    }
}

fn traveled(day: u32, minute: u16, miles: i64, driving: i64) -> JournalEntry {
    JournalEntry {
        action_kind: "travel".into(),
        day,
        minute,
        pace: Some(PaceId::Steady),
        diet: Some(DietId::Mixed),
        place: "Salt Lake City → Denver".into(),
        title: i18n::t("log.traveled"),
        message: String::new(),
        before: Stats::default(),
        after: Stats::default(),
        details: Vec::new(),
        resources: vec![
            change("play.miles", miles, miles + 600),
            change("play.driving_time", driving, driving + 60),
        ],
    }
}

#[test]
fn persisted_days_accumulate_actions_without_mutating_the_turn_history() {
    i18n::set_lang("en");
    let mut gs = GameState {
        day: 99,
        ..GameState::default()
    };
    gs.continuity.journal = vec![
        traveled(8, 540, 0, 0),
        traveled(9, 540, 1200, 120),
        traveled(8, 600, 600, 60),
    ];
    gs.continuity.turn_journal_start = Some(2);
    let before = serde_json::to_string(&gs.continuity).unwrap();
    let days = day::group(&gs.continuity.journal);
    assert_eq!(days.len(), 2);
    assert_eq!(
        days.iter().map(|day| day.number).collect::<Vec<_>>(),
        [8, 9]
    );
    assert_eq!(days[0].entries.len(), 2);
    assert_eq!(days[0].first().minute, 540);
    assert_eq!(days[0].last().minute, 600);
    assert_eq!(days[0].resources()[0], change("play.miles", 0, 1200));
    assert_eq!(gs.continuity.turn_entries(), &gs.continuity.journal[2..]);
    assert_eq!(serde_json::to_string(&gs.continuity).unwrap(), before);
}

#[test]
fn day_totals_use_actual_snapshots_after_spending_recovery_and_refunds() {
    i18n::set_lang("en");
    let mut first = traveled(8, 540, 1000, 300);
    first.before.supplies = 19;
    first.after.supplies = 20;
    first.before.sanity = 8;
    first.after.sanity = 9;
    first.resources.push(change("play.cash", 2000, 1800));
    first.resources.push(change("play.vehicle", 10000, 9800));

    let mut middle = traveled(8, 600, 1600, 360);
    middle.before = first.after.clone();
    middle.after = middle.before.clone();
    middle.after.supplies = 16;
    middle.after.sanity = 7;
    middle.resources.push(change("play.cash", 1800, 2000));
    middle.resources.push(change("ux.receipt", 0, 1));
    middle
        .resources
        .push(change("store.items.spare_tire.name", 1, 0));
    // Repeated snapshots must not charge the same wear twice in the daily summary.
    middle.resources.push(change("play.vehicle", 10000, 9800));

    let mut last = traveled(8, 660, 2200, 420);
    last.before = middle.after.clone();
    last.after = last.before.clone();
    last.after.supplies = 17;
    last.resources.push(change("play.vehicle", 9800, 9900));
    let entries = vec![first, middle, last];
    let day = day::group(&entries).remove(0);
    let stats = crate::components::ui::stat_card::resources(&day.first().before, &day.last().after);
    assert_eq!(
        stats,
        [change("ux.supplies", 19, 17), change("ux.sanity", 8, 7)]
    );
    assert_eq!(
        day.resources(),
        [
            change("play.miles", 1000, 2800),
            change("play.driving_time", 300, 480),
            change("play.vehicle", 10000, 9900),
            change("ux.receipt", 0, 1),
            change("store.items.spare_tire.name", 1, 0),
        ]
    );
    assert_eq!(day.entries[0].resources[2], change("play.cash", 2000, 1800));
    assert_eq!(day.entries[1].resources[2], change("play.cash", 1800, 2000));
}

#[test]
fn routine_travel_combines_without_hiding_crew_settings_or_route_changes() {
    i18n::set_lang("en");
    let mut entries: Vec<_> = (0_u16..8)
        .map(|hour| {
            traveled(
                8,
                540 + hour * 30,
                i64::from(hour) * 600,
                i64::from(hour) * 60,
            )
        })
        .collect();
    entries[2].action_kind = "care".into();
    entries[2].title = "A crew member needs help".into();
    entries[2].message = "Sam receives care and can help the crew again.".into();
    entries[2].details.push(("Sam".into(), "Traveling".into()));
    for entry in &mut entries[4..] {
        entry.pace = Some(PaceId::Heated);
        entry.diet = Some(DietId::Quiet);
    }
    entries[6].place = "At Denver".into();
    entries[7].place = "At Denver".into();
    entries[7]
        .details
        .push(("Harper".into(), "Left the crew".into()));
    let day = day::group(&entries).remove(0);
    let events = day.events();
    assert_eq!(
        events.iter().map(|event| event.count).collect::<Vec<_>>(),
        [2, 1, 1, 2, 1, 1]
    );
    assert_eq!(events[0].last.minute, 570);
    assert!(!events[1].routine);
    assert_eq!(
        events[1].first.details,
        [("Sam".into(), "Traveling".into())]
    );
    assert!(events[3].settings_changed);
    assert_eq!(events[3].first.pace, Some(PaceId::Heated));
    assert_eq!(events[3].first.diet, Some(DietId::Quiet));
    assert!(!events[4].settings_changed);
    assert_eq!(events[4].first.place, "At Denver");
    assert!(!events[5].routine);
    assert_eq!(day.places(), "Salt Lake City → Denver · At Denver");
}

#[test]
fn a_busy_day_keeps_its_earliest_snapshots_and_all_raw_actions() {
    i18n::set_lang("en");
    let entries: Vec<_> = (0_u16..60)
        .map(|action| {
            let mut entry = traveled(3, 480 + action, i64::from(action) * 10, i64::from(action));
            entry.resources = vec![
                change(
                    "play.miles",
                    i64::from(action) * 10,
                    i64::from(action + 1) * 10,
                ),
                change(
                    "play.driving_time",
                    i64::from(action),
                    i64::from(action + 1),
                ),
            ];
            entry
        })
        .collect();
    let day = day::group(&entries).remove(0);
    assert_eq!(day.entries.len(), 60);
    assert_eq!(day.events().len(), 1);
    assert_eq!(day.events()[0].count, 60);
    assert_eq!(day.resources()[0], change("play.miles", 0, 600));
    assert_eq!(day.resources()[1], change("play.driving_time", 0, 60));
}

#[derive(Clone, PartialEq, Properties)]
struct TestProps {
    content: Html,
}

#[function_component(JournalTest)]
fn journal_test(props: &TestProps) -> Html {
    props.content.clone()
}

fn rendered(gs: &GameState) -> String {
    block_on(
        LocalServerRenderer::<JournalTest>::with_props(TestProps {
            content: super::render(gs, &[]),
        })
        .render(),
    )
}

#[test]
fn journal_renders_one_card_per_day_with_totals_and_disclosed_action_evidence() {
    i18n::set_lang("en");
    let mut gs = GameState {
        day: 99,
        pace: PaceId::Blitz,
        diet: DietId::Doom,
        ..GameState::default()
    };
    gs.continuity.journal = vec![
        traveled(8, 540, 0, 0),
        traveled(8, 600, 600, 60),
        traveled(9, 540, 1200, 120),
    ];
    let html = rendered(&gs);
    assert_eq!(
        html.matches("class=\"journal-entry journal-day\"").count(),
        2
    );
    assert!(html.find("Day 9").unwrap() < html.find("Day 8").unwrap());
    assert!(!html.contains("Day 99"));
    assert_eq!(html.matches("class=\"journal-event\"").count(), 2);
    assert_eq!(html.matches("class=\"journal-raw-entry\"").count(), 3);
    assert_eq!(html.matches("<details class=\"journal-raw\">").count(), 2);
    assert!(html.contains("The Day's Events"));
    assert!(html.contains("+120.0"));
    assert!(html.contains("2 h driving"));
    assert!(!html.contains("data-stat=\"play.driving_time\""));
    assert!(html.contains("data-icon=\"steady\""));
    assert!(html.contains("data-icon=\"mixed\""));
    assert!(!html.contains("data-icon=\"blitz\""));
    assert!(!html.contains("data-icon=\"doom\""));
    assert_eq!(gs.continuity.journal.len(), 3);
}

#[test]
fn crew_outcomes_and_original_messages_remain_available_after_grouping() {
    i18n::set_lang("en");
    let mut entry = traveled(8, 660, 600, 60);
    entry.action_kind = "care".into();
    entry.title = "Care for Sam".into();
    entry.message = "Sam receives care and can help the crew again.".into();
    entry.details.push(("Sam".into(), "Traveling".into()));
    entry.after.supplies -= 2;
    let mut gs = GameState::default();
    gs.continuity.journal = vec![entry];
    let html = rendered(&gs);
    assert!(html.contains("Sam receives care and can help the crew again."));
    assert!(html.contains("Care for Sam"));
    assert!(html.contains("Traveling"));
    assert!(html.contains("data-icon=\"care\""));
    assert_eq!(html.matches("data-stat=\"ux.supplies\"").count(), 2);
    assert!(html.contains("-2"));
}
