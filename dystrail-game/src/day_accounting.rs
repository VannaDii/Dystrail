use crate::constants::{
    TRAVEL_CLASSIC_BASE_DISTANCE, TRAVEL_HISTORY_WINDOW, TRAVEL_PARTIAL_MIN_DISTANCE,
    TRAVEL_V2_BASE_DISTANCE,
};
use crate::journey::{DayRecord, TravelDayKind};
use crate::state::{GameState, PolicyKind, TravelProgressKind};

/// Aggregate metrics derived from recorded day history.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayLedgerMetrics {
    pub total_days: u32,
    pub travel_days: u32,
    pub partial_days: u32,
    pub non_travel_days: u32,
    pub total_miles: f32,
    pub travel_miles: f32,
    pub partial_miles: f32,
    pub non_travel_miles: f32,
}

impl DayLedgerMetrics {
    #[must_use]
    pub fn travel_ratio(self) -> f32 {
        if self.total_days == 0 {
            1.0
        } else {
            let traveled = f64::from(self.travel_days + self.partial_days);
            let total = f64::from(self.total_days);
            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::cast_possible_truncation)]
            {
                (traveled / total) as f32
            }
        }
    }
}

impl Default for DayLedgerMetrics {
    fn default() -> Self {
        Self {
            total_days: 0,
            travel_days: 0,
            partial_days: 0,
            non_travel_days: 0,
            total_miles: 0.0,
            travel_miles: 0.0,
            partial_miles: 0.0,
            non_travel_miles: 0.0,
        }
    }
}

/// Compute ledger metrics from immutable day records.
#[must_use]
pub fn compute_day_ledger_metrics(records: &[DayRecord]) -> DayLedgerMetrics {
    let mut metrics = DayLedgerMetrics::default();
    for record in records {
        metrics.total_days = metrics.total_days.saturating_add(1);
        metrics.total_miles += record.miles;
        match record.kind {
            TravelDayKind::Travel => {
                metrics.travel_days = metrics.travel_days.saturating_add(1);
                metrics.travel_miles += record.miles;
            }
            TravelDayKind::Partial => {
                metrics.partial_days = metrics.partial_days.saturating_add(1);
                metrics.partial_miles += record.miles;
            }
            TravelDayKind::NonTravel => {
                metrics.non_travel_days = metrics.non_travel_days.saturating_add(1);
                metrics.non_travel_miles += record.miles;
            }
        }
    }
    metrics
}

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

    if matches!(effective_kind, TravelDayKind::NonTravel) && enforce_endgame_stop_cap(state) {
        effective_kind = TravelDayKind::Partial;
        miles = partial_day_miles(state, miles);
        state.add_day_reason_tag("auto_cap");
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
        let credited = state.apply_travel_progress(miles, progress_kind);
        if credited > 0.0 {
            state.current_day_miles += credited;
            miles = credited;
        } else {
            miles = 0.0;
        }
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

    if state.endgame.active
        && matches!(effective_kind, TravelDayKind::Partial)
        && state.endgame.wear_shave_ratio < 1.0
    {
        apply_endgame_wear_shave(state);
    }

    (effective_kind, miles)
}

fn enforce_endgame_stop_cap(state: &GameState) -> bool {
    if !state.endgame.active {
        return false;
    }
    let window = usize::from(state.endgame.stop_cap_window.max(1));
    let max_full = usize::from(state.endgame.stop_cap_max_full);
    if max_full == 0 {
        return true;
    }
    let full_stops = state
        .recent_travel_days
        .iter()
        .rev()
        .take(window)
        .filter(|kind| matches!(kind, TravelDayKind::NonTravel))
        .count();
    full_stops >= max_full
}

fn apply_endgame_wear_shave(state: &mut GameState) {
    let shave = state.endgame.wear_shave_ratio;
    if !(0.0..1.0).contains(&shave) {
        return;
    }
    state.vehicle.wear *= shave;
    if state.vehicle.wear < 0.0 {
        state.vehicle.wear = 0.0;
    }
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
    if matches!(kind, TravelDayKind::Travel | TravelDayKind::Partial) {
        state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
    }
}

const fn adjust_counters_for_transition(
    state: &mut GameState,
    existing: TravelDayKind,
    next: TravelDayKind,
) {
    match (existing, next) {
        (TravelDayKind::Partial | TravelDayKind::NonTravel, TravelDayKind::Travel)
        | (TravelDayKind::NonTravel, TravelDayKind::Partial) => {
            state.rotation_travel_days = state.rotation_travel_days.saturating_add(1);
        }
        (TravelDayKind::Partial | TravelDayKind::Travel, TravelDayKind::NonTravel) => {
            state.rotation_travel_days = state.rotation_travel_days.saturating_sub(1);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameMode;
    use rand::{Rng, SeedableRng, rngs::StdRng};
    use serde_json;
    use std::collections::VecDeque;
    use std::convert::TryFrom;

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
        state.end_of_day();
        assert_eq!(state.travel_days, 0);
        assert_eq!(state.partial_travel_days, 0);
        assert_eq!(state.non_travel_days, 1);

        let (kind, _) = record_travel_day(&mut state, TravelDayKind::Partial, 3.0);
        assert_eq!(kind, TravelDayKind::Partial);
        state.end_of_day();
        assert_eq!(state.partial_travel_days, 1);
        assert_eq!(state.non_travel_days, 1);

        let (kind, _) = record_travel_day(&mut state, TravelDayKind::Travel, 10.0);
        assert_eq!(kind, TravelDayKind::Travel);
        assert!(state.traveled_today);
        assert!(!state.partial_traveled_today);
        state.end_of_day();
        assert_eq!(state.travel_days, 1);
        assert_eq!(state.partial_travel_days, 1);
        assert_eq!(state.non_travel_days, 1);
    }

    #[test]
    fn partial_day_miles_uses_policy_ratio() {
        let mut state = fresh_state();
        let base = 10.0;
        for ratio in [0.25_f32, 0.5, 0.75, 0.9] {
            state.distance_today = base;
            state.distance_today_raw = base;
            state.partial_distance_today = 0.0;
            state.journey_partial_ratio = ratio;
            let miles = partial_day_miles(&state, 0.0);
            let expected = (ratio.clamp(0.2, 0.95) * base).max(TRAVEL_PARTIAL_MIN_DISTANCE);
            assert!((miles - expected).abs() <= 1e-5);
        }
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

    #[test]
    fn ledger_metrics_align_with_distance() {
        let records = vec![
            DayRecord::new(0, TravelDayKind::Travel, 12.5),
            DayRecord::new(1, TravelDayKind::Partial, 6.0),
            DayRecord::new(2, TravelDayKind::NonTravel, 0.0),
        ];
        let metrics = compute_day_ledger_metrics(&records);
        assert_eq!(metrics.total_days, 3);
        assert!((metrics.total_miles - 18.5).abs() <= 1e-5);
        assert_eq!(metrics.travel_days, 1);
        assert_eq!(metrics.partial_days, 1);
        assert_eq!(metrics.non_travel_days, 1);
    }

    #[test]
    fn ledger_metrics_match_state_distance() {
        let mut state = fresh_state();
        record_travel_day(&mut state, TravelDayKind::Travel, 11.25);
        state.end_of_day();

        record_travel_day(&mut state, TravelDayKind::Partial, 4.75);
        state.end_of_day();

        record_travel_day(&mut state, TravelDayKind::NonTravel, 0.0);
        state.end_of_day();

        let metrics = state.ledger_metrics();
        assert!((metrics.total_miles - state.miles_traveled_actual).abs() <= 1e-5);
        assert_eq!(
            usize::try_from(metrics.total_days).unwrap(),
            state.day_records.len()
        );
    }

    #[test]
    fn ledger_travel_ratio_matches_manual_computation() {
        let mut rng = StdRng::seed_from_u64(0x5A5A);
        let mut records = Vec::new();
        for day in 0..40_u16 {
            let kind = match rng.random_range(0..3) {
                0 => TravelDayKind::Travel,
                1 => TravelDayKind::Partial,
                _ => TravelDayKind::NonTravel,
            };
            let miles = match kind {
                TravelDayKind::Travel => rng.random_range(5.0..18.0),
                TravelDayKind::Partial => rng.random_range(2.0..9.0),
                TravelDayKind::NonTravel => 0.0,
            };
            records.push(DayRecord::new(day, kind, miles));
        }

        let metrics = compute_day_ledger_metrics(&records);
        let mut travel = 0_u32;
        let mut partial = 0_u32;
        for record in &records {
            match record.kind {
                TravelDayKind::Travel => travel = travel.saturating_add(1),
                TravelDayKind::Partial => partial = partial.saturating_add(1),
                TravelDayKind::NonTravel => {}
            }
        }
        let total = u32::try_from(records.len()).expect("record count fits in u32");
        let manual_ratio = if total == 0 {
            1.0
        } else {
            let numerator = f64::from(travel + partial);
            let denominator = f64::from(total);
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            {
                (numerator / denominator) as f32
            }
        };
        assert!((metrics.travel_ratio() - manual_ratio).abs() <= 1e-5);
    }

    #[test]
    fn day_record_serialization_roundtrip_is_stable() {
        let records = vec![
            DayRecord::new(0, TravelDayKind::Travel, 9.25),
            DayRecord::new(1, TravelDayKind::Partial, 4.0),
            DayRecord::new(2, TravelDayKind::NonTravel, 0.0),
        ];
        let json = serde_json::to_string(&records).expect("serialize day ledger");
        let decoded: Vec<DayRecord> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.len(), records.len());
        for (expected, actual) in records.iter().zip(decoded.iter()) {
            assert_eq!(expected.kind, actual.kind);
            assert!((expected.miles - actual.miles).abs() <= 1e-6);
            assert_eq!(expected.tags, actual.tags);
        }
    }
}
