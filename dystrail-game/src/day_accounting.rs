use crate::constants::{
    TRAVEL_CLASSIC_BASE_DISTANCE, TRAVEL_HISTORY_WINDOW, TRAVEL_PARTIAL_MIN_DISTANCE,
    TRAVEL_V2_BASE_DISTANCE,
};
use crate::journey::TravelDayKind;
use crate::state::{GameState, PolicyKind, TravelProgressKind};

/// Record travel day details and update counters consistently.
pub fn record_travel_day(
    state: &mut GameState,
    kind: TravelDayKind,
    miles_earned: f32,
) -> (TravelDayKind, f32) {
    state.start_of_day();
    let mut effective_kind = kind;
    let mut miles = sanitize_miles(miles_earned);
    let suppress_stop_ratio = state.suppress_stop_ratio;

    if matches!(effective_kind, TravelDayKind::NonTravel)
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
            TravelDayKind::Travel => TravelProgressKind::Full,
            TravelDayKind::Partial | TravelDayKind::NonTravel => TravelProgressKind::Partial,
        };
        state.apply_travel_progress(miles, progress_kind);
        state.current_day_miles += miles;
    }

    match effective_kind {
        TravelDayKind::Travel => {
            state.traveled_today = true;
            state.partial_traveled_today = false;
        }
        TravelDayKind::Partial => {
            state.traveled_today = false;
            state.partial_traveled_today = true;
        }
        TravelDayKind::NonTravel => {
            state.traveled_today = false;
            state.partial_traveled_today = false;
        }
    }

    (effective_kind, miles)
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
        .filter(|kind| matches!(kind, TravelDayKind::NonTravel))
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

    let ratio = state.journey_partial_ratio.clamp(0.2, 0.95);

    let partial_today = state.partial_distance_today;
    if partial_today > 0.0 {
        return partial_today.max(TRAVEL_PARTIAL_MIN_DISTANCE);
    }
    let distance_today = state.distance_today;
    if distance_today > 0.0 {
        return (distance_today * ratio).max(TRAVEL_PARTIAL_MIN_DISTANCE);
    }

    let base = if state.features.travel_v2 {
        TRAVEL_V2_BASE_DISTANCE
    } else {
        TRAVEL_CLASSIC_BASE_DISTANCE
    };
    (base * ratio).max(TRAVEL_PARTIAL_MIN_DISTANCE)
}

const fn apply_initial_counters(state: &mut GameState, kind: TravelDayKind) {
    match kind {
        TravelDayKind::Travel => {
            state.travel_days = state.travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        TravelDayKind::Partial => {
            state.partial_travel_days = state.partial_travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        TravelDayKind::NonTravel => {
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
        (TravelDayKind::Partial, TravelDayKind::Travel) => {
            state.partial_travel_days = state.partial_travel_days.saturating_sub(1);
            state.travel_days = state.travel_days.saturating_add(1);
        }
        (TravelDayKind::NonTravel, TravelDayKind::Partial) => {
            state.non_travel_days = state.non_travel_days.saturating_sub(1);
            state.partial_travel_days = state.partial_travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        (TravelDayKind::NonTravel, TravelDayKind::Travel) => {
            state.non_travel_days = state.non_travel_days.saturating_sub(1);
            state.travel_days = state.travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        (TravelDayKind::Partial, TravelDayKind::NonTravel) => {
            state.partial_travel_days = state.partial_travel_days.saturating_sub(1);
            state.non_travel_days = state.non_travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_sub(1);
        }
        (TravelDayKind::Travel, TravelDayKind::Partial) => {
            state.travel_days = state.travel_days.saturating_sub(1);
            state.partial_travel_days = state.partial_travel_days.saturating_add(1);
        }
        (TravelDayKind::Travel, TravelDayKind::NonTravel) => {
            state.travel_days = state.travel_days.saturating_sub(1);
            state.non_travel_days = state.non_travel_days.saturating_add(1);
            state.rotation_travel_days = state.rotation_travel_days.saturating_sub(1);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameMode;
    use std::collections::VecDeque;

    fn fresh_state() -> GameState {
        let mut state = GameState::default();
        state.features.travel_v2 = true;
        state
    }

    #[test]
    fn record_travel_day_applies_transitions() {
        let mut state = fresh_state();
        let (kind, _) = record_travel_day(&mut state, TravelDayKind::NonTravel, 0.0);
        assert_eq!(kind, TravelDayKind::NonTravel);
        assert_eq!(state.non_travel_days, 1);

        let (kind, _) = record_travel_day(&mut state, TravelDayKind::Partial, 3.0);
        assert_eq!(kind, TravelDayKind::Partial);
        assert_eq!(state.partial_travel_days, 1);
        assert_eq!(state.non_travel_days, 0);

        let (kind, _) = record_travel_day(&mut state, TravelDayKind::Travel, 10.0);
        assert_eq!(kind, TravelDayKind::Travel);
        assert_eq!(state.travel_days, 1);
        assert_eq!(state.partial_travel_days, 0);
        assert!(state.traveled_today);
        assert!(!state.partial_traveled_today);
    }

    #[test]
    fn partial_day_miles_uses_policy_ratio() {
        let mut state = fresh_state();
        state.distance_today = 10.0;
        state.journey_partial_ratio = 0.75;
        let miles = partial_day_miles(&state, 0.0);
        assert!((miles - 7.5).abs() <= f32::EPSILON);
    }

    #[test]
    fn partial_day_miles_falls_back_to_deltas() {
        let mut state = fresh_state();
        state.partial_distance_today = 4.0;
        let miles = partial_day_miles(&state, 0.0);
        assert!((miles - 4.0).abs() <= f32::EPSILON);

        state.partial_distance_today = 0.0;
        state.distance_today = 6.0;
        let computed = partial_day_miles(&state, 0.0);
        assert!(computed >= TRAVEL_PARTIAL_MIN_DISTANCE);

        state.distance_today = 0.0;
        state.features.travel_v2 = false;
        let computed = partial_day_miles(&state, 0.0);
        assert!(computed >= TRAVEL_PARTIAL_MIN_DISTANCE);
    }

    #[test]
    fn enforce_ratio_floor_checks_recent_history() {
        let mut state = fresh_state();
        state.recent_travel_days =
            VecDeque::from(vec![TravelDayKind::NonTravel; TRAVEL_HISTORY_WINDOW]);
        assert!(enforce_ratio_floor(&state));

        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Conservative);
        state.recent_travel_days =
            VecDeque::from(vec![TravelDayKind::NonTravel; TRAVEL_HISTORY_WINDOW]);
        assert!(enforce_ratio_floor(&state));
    }
}
