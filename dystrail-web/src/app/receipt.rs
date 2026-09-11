//! Actual non-stat resource changes, including spare parts, cash and repair damage.
use crate::{game::GameState, i18n};
#[must_use]
pub fn resource_details(before: &GameState, after: &GameState) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    for (key, a, b) in [
        (
            "store.items.spare_tire.name",
            before.inventory.spares.tire,
            after.inventory.spares.tire,
        ),
        (
            "store.items.battery.name",
            before.inventory.spares.battery,
            after.inventory.spares.battery,
        ),
        (
            "store.items.alternator.name",
            before.inventory.spares.alt,
            after.inventory.spares.alt,
        ),
        (
            "store.items.fuel_pump.name",
            before.inventory.spares.pump,
            after.inventory.spares.pump,
        ),
    ] {
        if a != b {
            rows.push((i18n::t(key), format!("{a} → {b} ({:+})", b - a)));
        }
    }
    if before.budget_cents != after.budget_cents {
        rows.push((
            i18n::t("play.cash"),
            format!(
                "{} → {}",
                i18n::fmt_currency(before.budget_cents),
                i18n::fmt_currency(after.budget_cents)
            ),
        ));
    }
    if (before.vehicle.health - after.vehicle.health).abs() > 0.01 {
        rows.push((
            i18n::t("play.vehicle"),
            format!(
                "{:.2}% → {:.2}%",
                before.vehicle.health, after.vehicle.health
            ),
        ));
    }
    if before.day != after.day {
        rows.push((
            i18n::t("play.elapsed"),
            after.day.saturating_sub(before.day).to_string(),
        ));
    }
    let miles_before = crate::game::route::physical_miles(before);
    let miles_after = crate::game::route::physical_miles(after);
    if miles_after - miles_before > 0.05 {
        rows.push((
            i18n::t("play.miles"),
            format!("{miles_before:.1} → {miles_after:.1}"),
        ));
    }
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
#[cfg(test)]
mod tests {
    #[test]
    fn consumed_pump_is_visible_even_when_stats_are_identical() {
        crate::i18n::set_lang("en");
        let mut before = crate::game::GameState::default();
        before.inventory.spares.pump = 1;
        let mut after = before.clone();
        after.inventory.spares.pump = 0;
        let rows = super::resource_details(&before, &after);
        assert!(
            rows.iter()
                .any(|(k, v)| k == "Fuel Pump" && v == "1 → 0 (-1)")
        );
    }
}
