//! Reviewed A-version prose bound to existing game state; no simulation decisions.
use crate::{
    game::{GameState, Weather, activities::Activity, repairs::RepairChoice, vehicle::Part},
    i18n,
};

#[must_use]
pub fn text(family: &str, field: &str) -> String {
    i18n::t(&format!("workshop.{family}.{field}"))
}

#[must_use]
pub fn named(family: &str, field: &str, name: &str) -> String {
    i18n::tr(
        &format!("workshop.{family}.{field}"),
        Some(&std::collections::BTreeMap::from([("name", name)])),
    )
}

#[must_use]
pub fn care(reason: u8) -> String {
    format!("CARE-{:02}", u16::from(reason) + 1)
}

#[must_use]
pub fn care_outcome(key: &str) -> &'static str {
    match key {
        "journey.care_helped" => "outcomes.helped",
        "journey.care_sheltered" => "outcomes.sheltered",
        "journey.care_lost" => "outcomes.companion_lost",
        "journey.player_lost" => "outcomes.player_lost",
        _ => "outcomes.deferred",
    }
}

#[must_use]
pub const fn repair(part: Part) -> &'static str {
    match part {
        Part::Tire => "REPAIR-TIRE",
        Part::Battery => "REPAIR-BATTERY",
        Part::Alternator => "REPAIR-ALTERNATOR",
        Part::FuelPump => "REPAIR-FUELPUMP",
    }
}

#[must_use]
pub const fn repair_outcome(choice: RepairChoice, roadside: bool) -> &'static str {
    match choice {
        RepairChoice::Onboard => "choices.c0.outcome",
        RepairChoice::Purchase if roadside => "choices.c1.roadside_outcome",
        RepairChoice::Purchase => "choices.c1.outcome",
        RepairChoice::Barter => "choices.c2.outcome",
        RepairChoice::Radio => "choices.c3.outcome",
    }
}

#[must_use]
pub const fn activity(action: Activity) -> &'static str {
    match action {
        Activity::Forage => "ACT-FORAGE",
        Activity::Glean => "ACT-GLEAN",
        Activity::WorkSupplies => "ACT-FOODWORK",
        Activity::WorkCash => "ACT-CASHWORK",
    }
}

#[must_use]
pub const fn trade(kind: u8) -> &'static str {
    match kind {
        0 => "ACT-BARTERTIRE",
        1 => "ACT-BARTERBATTERY",
        _ => "ACT-BARTERSUPPLIES",
    }
}

#[must_use]
pub const fn weather(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "COND-CLEAR",
        Weather::Storm => "COND-STORM",
        Weather::HeatWave => "COND-HEAT",
        Weather::ColdSnap => "COND-COLD",
        Weather::Smoke => "COND-SMOKE",
    }
}

#[must_use]
pub fn opening(gs: &GameState, field: &str) -> String {
    let persona = gs
        .persona_id
        .as_deref()
        .unwrap_or("journalist")
        .to_uppercase();
    text(&format!("OPEN-{persona}"), field)
}

#[must_use]
pub fn hearing(gs: &GameState, field: &str) -> String {
    text(
        if gs.mode.is_deep() {
            "HEARING-D"
        } else {
            "HEARING-C"
        },
        field,
    )
}

#[must_use]
pub const fn order(order: crate::game::exec_orders::ExecOrder) -> &'static str {
    use crate::game::exec_orders::ExecOrder;
    match order {
        ExecOrder::Shutdown => "ORDER-SHUTDOWN",
        ExecOrder::TravelBanLite => "ORDER-MILITARIZE",
        ExecOrder::BookPanic => "ORDER-GAG",
        ExecOrder::TariffTsunami => "ORDER-TARIFFS",
        ExecOrder::DoEEliminated => "ORDER-TAXCUTS",
        ExecOrder::WarDeptReorg => "ORDER-DEREGULATE",
    }
}

/// Summarize only the procedure the existing hearing actually completed.
#[must_use]
pub fn hearing_outcome(
    gs: &GameState,
    cfg: &crate::game::BossConfig,
    outcome: crate::game::boss::BossOutcome,
) -> String {
    if outcome == crate::game::boss::BossOutcome::Exhausted {
        return hearing(gs, "exhausted");
    }
    let mut lines = Vec::new();
    if cfg.rounds == 3 {
        for round in 0..3 {
            lines.push(hearing(gs, &format!("transitions.r{round}")));
        }
    }
    lines.push(hearing(gs, "before_vote"));
    lines.push(i18n::t(if gs.boss.outcome.victory {
        "journey.vote_won"
    } else {
        "journey.vote_lost"
    }));
    lines.join(" ")
}
