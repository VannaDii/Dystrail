//! Narrative for completed engine events; all decisions and costs remain in the engine.
use super::workshop;
use crate::{
    game::{
        GameState,
        state::{CrossingDetourReason, CrossingOutcomeTelemetry},
    },
    i18n,
};

#[must_use]
pub fn message(line: &str) -> String {
    if line == "log.allies.gone" {
        return workshop::text("ALLY-01", "final_ally");
    }
    let condition = match line {
        "log.disease.hit" => Some(("COND-ILLNESS", "onset")),
        "log.disease.tick" => Some(("COND-ILLNESS", "continuing")),
        "log.disease.recover" => Some(("COND-ILLNESS", "recovery")),
        "log.supplies_depleted" => Some(("COND-HUNGER", "onset")),
        "log.starvation.tick" => Some(("COND-HUNGER", "continuing")),
        "log.starvation.relief" => Some(("COND-HUNGER", "recovery")),
        "log.weather.exposure" => Some(("COND-COLDEXPOSURE", "onset")),
        "log.weather.heatstroke" => Some(("COND-HEATEXPOSURE", "onset")),
        _ => None,
    };
    if let Some((family, field)) = condition {
        return workshop::text(family, field);
    }
    for (prefix, field) in [("exec.start.", "activation"), ("exec.end.", "expiration")] {
        if let Some(key) = line.strip_prefix(prefix)
            && let Some(order) = crate::game::exec_orders::ExecOrder::ALL
                .iter()
                .find(|o| o.key() == key)
        {
            return workshop::text(workshop::order(*order), field);
        }
    }
    i18n::log_message(line)
}

#[must_use]
pub fn is_crossing(line: &str) -> bool {
    line.starts_with("log.crossing.") || line.starts_with("crossing.result.")
}

#[must_use]
pub fn crossings(before: &GameState, after: &GameState) -> Vec<String> {
    after
        .crossing_events
        .iter()
        .skip(before.crossing_events.len())
        .enumerate()
        .map(|(offset, event)| {
            let index = usize::try_from(before.crossings_completed).unwrap_or(0) + offset;
            let family = match (index, after.mode.is_deep()) {
                (0, _) => "CROSS-01",
                (1, false) => "CROSS-02C",
                (1, true) => "CROSS-02D",
                _ => "CROSS-03",
            };
            let field = match event.outcome {
                CrossingOutcomeTelemetry::Passed if event.bribe_success == Some(true) => {
                    "bribe_success"
                }
                CrossingOutcomeTelemetry::Passed => "passage",
                CrossingOutcomeTelemetry::Detoured
                    if event.detour_reason == Some(CrossingDetourReason::CheckpointDenied)
                        && index == 0 =>
                {
                    "refused_passage"
                }
                CrossingOutcomeTelemetry::Detoured => "diversion",
                CrossingOutcomeTelemetry::Failed => "terminal_failure",
            };
            // Current permit handling does not consume a Receipt. The draft's consumption
            // claim must not be imported into the narrative of a successful passage.
            let mut lines = vec![workshop::text(family, "setup")];
            if event.bribe_success == Some(false) {
                lines.push(workshop::text(family, "outcomes.bribe_failure"));
            }
            lines.push(workshop::text(family, &format!("outcomes.{field}")));
            lines.join(" ")
        })
        .collect()
}

/// Describe a weather change only after the engine records it.
#[must_use]
pub fn weather(before: &GameState, after: &GameState) -> Vec<String> {
    if before.weather_state.today == after.weather_state.today {
        return Vec::new();
    }
    vec![
        workshop::text(workshop::weather(before.weather_state.today), "recovery"),
        workshop::text(workshop::weather(after.weather_state.today), "onset"),
    ]
}
