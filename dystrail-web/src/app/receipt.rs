//! Actual non-stat resource changes, including spare parts, cash and repair damage.
use crate::{game::GameState, i18n};
#[must_use]
pub fn label(key: &str) -> String {
    i18n::t(if key == "play.cash" { "ux.cash" } else { key })
}

#[must_use]
pub fn resource_changes(
    before: &GameState,
    after: &GameState,
) -> Vec<crate::game::journal::ResourceChange> {
    use crate::game::journal::ResourceChange;
    let condition =
        |n: f32| i64::from(crate::game::numbers::round_f64_to_i32(f64::from(n) * 100.0));
    let miles = |gs: &GameState| {
        i64::from(crate::game::numbers::round_f64_to_i32(
            f64::from(crate::game::route::physical_miles(gs)) * 10.0,
        ))
    };
    [
        ("play.cash", before.budget_cents, after.budget_cents),
        ("play.elapsed", i64::from(before.day), i64::from(after.day)),
        ("play.miles", miles(before), miles(after)),
        (
            "play.driving_time",
            i64::from(before.continuity.driving_minutes_total),
            i64::from(after.continuity.driving_minutes_total),
        ),
        (
            "ux.receipt",
            i64::try_from(before.receipts.len()).unwrap_or(0),
            i64::try_from(after.receipts.len()).unwrap_or(0),
        ),
        (
            "play.vehicle",
            condition(before.vehicle.health),
            condition(after.vehicle.health),
        ),
        (
            "store.items.spare_tire.name",
            i64::from(before.inventory.spares.tire),
            i64::from(after.inventory.spares.tire),
        ),
        (
            "store.items.battery.name",
            i64::from(before.inventory.spares.battery),
            i64::from(after.inventory.spares.battery),
        ),
        (
            "store.items.alternator.name",
            i64::from(before.inventory.spares.alt),
            i64::from(after.inventory.spares.alt),
        ),
        (
            "store.items.fuel_pump.name",
            i64::from(before.inventory.spares.pump),
            i64::from(after.inventory.spares.pump),
        ),
    ]
    .into_iter()
    .filter(|(_, a, b)| a != b)
    .map(|(key, before, after)| ResourceChange {
        key: key.into(),
        before,
        after,
    })
    .collect()
}

/// Actual time behind the wheel, independent of calendar or overnight changes.
#[must_use]
pub fn driving_duration(minutes: u32) -> String {
    let hours = minutes / 60;
    let remainder = minutes % 60;
    let hour_label = i18n::fmt_number(f64::from(hours));
    let minute_label = i18n::fmt_number(f64::from(remainder));
    i18n::tr(
        match (hours, remainder) {
            (0, _) => "play.driving_minutes",
            (_, 0) => "play.driving_hours",
            _ => "play.driving_hours_minutes",
        },
        Some(&std::collections::BTreeMap::from([
            ("hours", hour_label.as_str()),
            ("minutes", minute_label.as_str()),
        ])),
    )
}

#[must_use]
pub fn delta(change: &crate::game::journal::ResourceChange) -> String {
    let delta = change.after - change.before;
    match change.key.as_str() {
        "play.cash" => format!(
            "{}{}",
            if delta >= 0 { "+" } else { "−" },
            i18n::fmt_currency(delta.abs())
        ),
        "play.vehicle" => format!(
            "{:+.2}%",
            f64::from(i32::try_from(delta).unwrap_or(0)) / 100.0
        ),
        "play.miles" => format!(
            "{:+.1}",
            f64::from(i32::try_from(delta).unwrap_or(0)) / 10.0
        ),
        _ => format!("{delta:+}"),
    }
}
#[must_use]
pub fn resource_details(before: &GameState, after: &GameState) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    for member in &after.party.members {
        let previous = before
            .party
            .members
            .iter()
            .find(|m| m.persona == member.persona);
        let changed = previous.is_some_and(|m| m.status != member.status);
        let was_unwell = before
            .continuity
            .crew_care
            .strain
            .get(&member.persona)
            .copied()
            .unwrap_or(0)
            > 0;
        let unwell = after
            .continuity
            .crew_care
            .strain
            .get(&member.persona)
            .copied()
            .unwrap_or(0)
            > 0;
        if changed || was_unwell != unwell {
            let key = match member.status {
                crate::game::party::MemberStatus::Dead => "crew.dead",
                crate::game::party::MemberStatus::Departed => "crew.departed",
                crate::game::party::MemberStatus::Active => {
                    if unwell {
                        "journey.struggling"
                    } else {
                        "crew.active"
                    }
                }
            };
            rows.push((member.name.clone(), i18n::t(key)));
        }
    }
    rows
}
#[must_use]
pub fn value(key: &str, value: i64) -> String {
    let number = f64::from(i32::try_from(value).unwrap_or(0));
    match key {
        "play.cash" => i18n::fmt_currency(value),
        "play.vehicle" => format!("{:.2}%", number / 100.0),
        "play.miles" => format!("{:.1}", number / 10.0),
        _ => i18n::fmt_number(number),
    }
}

/// Narrative details contain crew status, never a second numeric stat readout.
#[must_use]
pub fn narrative_detail(name: &str) -> bool {
    ![
        "journey.when",
        "play.miles",
        "play.elapsed",
        "play.driving_time",
        "play.cash",
        "play.vehicle",
        "store.items.spare_tire.name",
        "store.items.battery.name",
        "store.items.alternator.name",
        "store.items.fuel_pump.name",
    ]
    .iter()
    .any(|key| name == i18n::t(key))
}

#[cfg(test)]
mod tests {
    #[test]
    fn mileage_days_and_spares_use_typed_changes() {
        crate::i18n::set_lang("en");
        let mut before = crate::game::GameState::default();
        before.trail_distance = crate::game::route::for_state(&before).unwrap().total_miles;
        before.inventory.spares.pump = 1;
        before.miles_traveled_actual = 1792.2;
        before.continuity.driving_minutes_total = 420;
        let mut after = before.clone();
        after.inventory.spares.pump = 0;
        after.miles_traveled_actual = 1812.2;
        after.day += 1;
        after.continuity.driving_minutes_total = 480;
        let rows = super::resource_changes(&before, &after);
        let miles = rows.iter().find(|r| r.key == "play.miles").unwrap();
        assert_eq!(super::delta(miles), "+20.0");
        assert_eq!(super::value(&miles.key, miles.before), "1792.2");
        assert_eq!(super::value(&miles.key, miles.after), "1812.2");
        assert!(
            rows.iter()
                .any(|r| r.key == "play.elapsed" && r.after - r.before == 1)
        );
        assert!(
            rows.iter()
                .any(|r| r.key == "play.driving_time" && r.after - r.before == 60)
        );
        assert!(
            rows.iter()
                .any(|r| r.key == "store.items.fuel_pump.name" && r.before == 1 && r.after == 0)
        );
        assert!(super::resource_details(&before, &after).is_empty());
    }

    #[test]
    fn driving_durations_preserve_partial_hours() {
        crate::i18n::set_lang("en");
        assert_eq!(super::driving_duration(33), "33 min driving");
        assert_eq!(super::driving_duration(60), "1 h driving");
        assert_eq!(super::driving_duration(93), "1 h 33 min driving");
    }
}
