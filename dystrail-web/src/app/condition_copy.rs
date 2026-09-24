//! Narrate recorded condition changes without applying costs or interrupting travel.
use super::visual_content;
use crate::{
    game::{GameState, Weather, journal::JournalEntry},
    i18n,
};

fn weather_family(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "COND-CLEAR",
        Weather::ColdSnap => "COND-COLD",
        Weather::HeatWave => "COND-HEAT",
        Weather::Smoke => "COND-SMOKE",
        Weather::Storm => "COND-STORM",
    }
}

fn events(before: &GameState, after: &GameState) -> Vec<(&'static str, &'static str)> {
    let mut events = Vec::new();
    if before.weather_state.today != after.weather_state.today {
        events.push((weather_family(before.weather_state.today), "relief"));
        events.push((weather_family(after.weather_state.today), "desc"));
    } else if after.day > before.day {
        events.push((weather_family(after.weather_state.today), "continuing"));
    }
    for (family, was, now) in [
        (
            "COND-HUNGER",
            before.stats.supplies <= 0,
            after.stats.supplies <= 0,
        ),
        (
            "COND-ILLNESS",
            before.illness_days_remaining > 0,
            after.illness_days_remaining > 0,
        ),
    ] {
        if !was && now {
            events.push((family, "desc"));
        } else if was && !now {
            events.push((family, "relief"));
        } else if now && after.day > before.day {
            events.push((family, "continuing"));
        }
    }
    for (family, log, previous, current) in [
        (
            "COND-COLDEXPOSURE",
            "log.weather.exposure",
            before.exposure_streak_cold,
            after.exposure_streak_cold,
        ),
        (
            "COND-HEATEXPOSURE",
            "log.weather.heatstroke",
            before.exposure_streak_heat,
            after.exposure_streak_heat,
        ),
    ] {
        if after
            .logs
            .iter()
            .skip(before.logs.len())
            .any(|line| line == log)
        {
            events.push((family, if previous < 3 { "desc" } else { "continuing" }));
        } else if previous >= 3 && current == 0 {
            events.push((family, "relief"));
        }
    }
    events
}

pub fn record(before: &GameState, after: &mut GameState, day: u32, minute: u16, place: &str) {
    if after.continuity.visual_content.edition != visual_content::EDITION {
        return;
    }
    for (family, field) in events(before, after) {
        let unit = visual_content::service_unit(after, family, "condition", 0);
        after
            .continuity
            .visual_content
            .selections
            .entry(format!("{family}/condition/0"))
            .or_insert(unit.clone());
        after.continuity.journal.push(JournalEntry {
            action_kind: "travel".into(),
            day,
            minute,
            pace: Some(before.pace),
            diet: Some(before.diet),
            place: place.into(),
            title: i18n::t(&format!("encounter_copy.{unit}.name")),
            message: i18n::t(&format!("encounter_copy.{unit}.{field}")),
            // The primary action receipt accounts for all costs. These entries are narrative only.
            before: after.stats.clone(),
            after: after.stats.clone(),
            resources: vec![],
            details: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conditions_follow_transitions_and_actual_exposure_damage() {
        let before = GameState::default();
        let mut after = before.clone();
        assert!(events(&before, &after).is_empty());
        after.weather_state.today = Weather::ColdSnap;
        after.exposure_streak_cold = 2;
        assert_eq!(
            events(&before, &after),
            vec![("COND-CLEAR", "relief"), ("COND-COLD", "desc")]
        );
        let mut before = after.clone();
        after.exposure_streak_cold = 3;
        after.logs.push("log.weather.exposure".into());
        assert_eq!(events(&before, &after), vec![("COND-COLDEXPOSURE", "desc")]);
        before = after.clone();
        after.exposure_streak_cold = 4;
        assert!(
            events(&before, &after).is_empty(),
            "lockout does not invent damage"
        );
        after.exposure_streak_cold = 0;
        assert_eq!(
            events(&before, &after),
            vec![("COND-COLDEXPOSURE", "relief")]
        );
        before = GameState::default();
        before.stats.supplies = 1;
        after = before.clone();
        after.stats.supplies = 0;
        after.illness_days_remaining = 2;
        assert_eq!(
            events(&before, &after),
            vec![("COND-HUNGER", "desc"), ("COND-ILLNESS", "desc")]
        );
        before = after.clone();
        after.stats.supplies = 1;
        after.illness_days_remaining = 0;
        assert_eq!(
            events(&before, &after),
            vec![("COND-HUNGER", "relief"), ("COND-ILLNESS", "relief")]
        );
    }

    #[test]
    fn condition_journal_does_not_change_simulation_or_duplicate_costs() {
        let mut before = GameState::default();
        before.stats.supplies = 1;
        let mut after = before.clone();
        after.stats.supplies = 0;
        after.day += 1;
        after.continuity.visual_content.edition = visual_content::EDITION;
        let mut expected = serde_json::to_value(&after).unwrap();
        record(&before, &mut after, 2, 480, "On the road");
        assert_eq!(after.continuity.journal.len(), 2);
        for entry in &after.continuity.journal {
            assert_eq!(entry.before, entry.after);
            assert!(entry.resources.is_empty());
            assert!(!entry.message.contains("encounter_copy."));
        }
        let restored: GameState =
            serde_json::from_value(serde_json::to_value(&after).unwrap()).unwrap();
        assert_eq!(restored.continuity.journal, after.continuity.journal);
        let mut actual = serde_json::to_value(&after).unwrap();
        for key in ["journal", "visual_content"] {
            actual.as_object_mut().unwrap().remove(key);
            expected.as_object_mut().unwrap().remove(key);
        }
        assert_eq!(actual, expected);
    }
}
