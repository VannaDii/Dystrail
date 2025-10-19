use serde::{Deserialize, Serialize};

use crate::constants::{
    TRAVEL_CLASSIC_BASE_DISTANCE, TRAVEL_HISTORY_WINDOW, TRAVEL_PARTIAL_MIN_DISTANCE,
    TRAVEL_PARTIAL_RATIO, TRAVEL_V2_BASE_DISTANCE,
};
use crate::state::{GameState, PolicyKind, TravelProgressKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TravelDayKind {
    Full,
    Partial,
    Stop,
}

/// Record travel day details and update counters consistently.
pub fn record_travel_day(
    state: &mut GameState,
    kind: TravelDayKind,
    miles_earned: f32,
) -> TravelDayKind {
    state.start_of_day();
    let mut effective_kind = kind;
    let mut miles = sanitize_miles(miles_earned);
    let suppress_stop_ratio = state.suppress_stop_ratio;

    if matches!(effective_kind, TravelDayKind::Stop)
        && !suppress_stop_ratio
        && enforce_ratio_floor(state)
    {
        effective_kind = TravelDayKind::Partial;
        miles = partial_day_miles(state, miles);
        state.add_day_reason_tag("stop_cap");
    }

    match state.current_day_kind {
        None => {
            apply_initial_counters(state, effective_kind);
            state.current_day_kind = Some(effective_kind);
        }
        Some(existing) if existing != effective_kind => {
            adjust_counters_for_transition(state, existing, effective_kind);
            state.current_day_kind = Some(effective_kind);
        }
        _ => {}
    }

    if miles > 0.0 {
        let progress_kind = match effective_kind {
            TravelDayKind::Full => TravelProgressKind::Full,
            TravelDayKind::Partial | TravelDayKind::Stop => TravelProgressKind::Partial,
        };
        state.apply_travel_progress(miles, progress_kind);
        state.current_day_miles += miles;
    }

    match effective_kind {
        TravelDayKind::Full => {
            state.traveled_today = true;
            state.partial_traveled_today = false;
        }
        TravelDayKind::Partial => {
            state.traveled_today = false;
            state.partial_traveled_today = true;
        }
        TravelDayKind::Stop => {
            state.traveled_today = false;
            state.partial_traveled_today = false;
        }
    }

    effective_kind
}

const fn sanitize_miles(miles: f32) -> f32 {
    if miles.is_finite() {
        miles.max(0.0)
    } else {
        0.0
    }
}

fn enforce_ratio_floor(state: &GameState) -> bool {
    let window = TRAVEL_HISTORY_WINDOW.saturating_sub(1);
    if window == 0 {
        return false;
    }

    let stop_cap = ratio_stop_limit(state);
    let recent_stops = state
        .recent_travel_days
        .iter()
        .rev()
        .take(window)
        .filter(|kind| matches!(kind, TravelDayKind::Stop))
        .count();
    recent_stops >= stop_cap
}

const fn ratio_stop_limit(state: &GameState) -> usize {
    if state.mode.is_deep() && matches!(state.policy, Some(PolicyKind::Conservative)) {
        2
    } else {
        1
    }
}

pub(crate) fn partial_day_miles(state: &GameState, miles: f32) -> f32 {
    if miles > 0.0 {
        return miles;
    }

    let partial_today = state.partial_distance_today;
    if partial_today > 0.0 {
        return partial_today.max(TRAVEL_PARTIAL_MIN_DISTANCE);
    }
    let distance_today = state.distance_today;
    if distance_today > 0.0 {
        return (distance_today * TRAVEL_PARTIAL_RATIO).max(TRAVEL_PARTIAL_MIN_DISTANCE);
    }

    let base = if state.features.travel_v2 {
        TRAVEL_V2_BASE_DISTANCE
    } else {
        TRAVEL_CLASSIC_BASE_DISTANCE
    };
    (base * TRAVEL_PARTIAL_RATIO).max(TRAVEL_PARTIAL_MIN_DISTANCE)
}

const fn apply_initial_counters(state: &mut GameState, kind: TravelDayKind) {
    match kind {
        TravelDayKind::Full => {
            state.travel_days = state.travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        TravelDayKind::Partial => {
            state.partial_travel_days = state.partial_travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        TravelDayKind::Stop => {
            state.non_travel_days = state.non_travel_days.saturating_add(1);
        }
    }
}

const fn adjust_counters_for_transition(
    state: &mut GameState,
    existing: TravelDayKind,
    next: TravelDayKind,
) {
    match (existing, next) {
        (TravelDayKind::Partial, TravelDayKind::Full) => {
            state.partial_travel_days = state.partial_travel_days.saturating_sub(1);
            state.travel_days = state.travel_days.saturating_add(1);
        }
        (TravelDayKind::Stop, TravelDayKind::Partial) => {
            state.non_travel_days = state.non_travel_days.saturating_sub(1);
            state.partial_travel_days = state.partial_travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        (TravelDayKind::Stop, TravelDayKind::Full) => {
            state.non_travel_days = state.non_travel_days.saturating_sub(1);
            state.travel_days = state.travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        (TravelDayKind::Partial, TravelDayKind::Stop) => {
            state.partial_travel_days = state.partial_travel_days.saturating_sub(1);
            state.non_travel_days = state.non_travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_sub(1);
        }
        (TravelDayKind::Full, TravelDayKind::Partial) => {
            state.travel_days = state.travel_days.saturating_sub(1);
            state.partial_travel_days = state.partial_travel_days.saturating_add(1);
        }
        (TravelDayKind::Full, TravelDayKind::Stop) => {
            state.travel_days = state.travel_days.saturating_sub(1);
            state.non_travel_days = state.non_travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_sub(1);
        }
        _ => {}
    }
}
