//! Records actual movement and time in the current and completed driving days.

/// The day's ledger stays open across individual hours and stationary actions.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct OpenDayLedger {
    pub current_day_record: Option<DayRecord>,
    pub current_day_kind: Option<TravelDayKind>,
    pub current_day_reason_tags: Vec<String>,
    pub current_day_miles: f32,
}

use crate::journey::{DayRecord, TravelDayKind};
use crate::numbers::clamp_f64_to_f32;
use crate::state::{GameState, TravelProgressKind};

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
        compute_ratio(self.travel_days + self.partial_days, self.total_days)
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
    let mut miles = [0.0_f64; 3];
    for record in records {
        metrics.total_days = metrics.total_days.saturating_add(1);
        let bucket = match record.kind {
            TravelDayKind::Travel => {
                metrics.travel_days = metrics.travel_days.saturating_add(1);
                0
            }
            TravelDayKind::Partial => {
                metrics.partial_days = metrics.partial_days.saturating_add(1);
                1
            }
            TravelDayKind::NonTravel => {
                metrics.non_travel_days = metrics.non_travel_days.saturating_add(1);
                2
            }
        };
        miles[bucket] += f64::from(record.miles);
    }
    metrics.total_miles = clamp_f64_to_f32(miles.iter().sum());
    metrics.travel_miles = clamp_f64_to_f32(miles[0]);
    metrics.partial_miles = clamp_f64_to_f32(miles[1]);
    metrics.non_travel_miles = clamp_f64_to_f32(miles[2]);
    metrics
}

fn compute_ratio(numerator: u32, denominator: u32) -> f32 {
    if denominator == 0 {
        return 1.0;
    }
    let ratio = f64::from(numerator) / f64::from(denominator);
    clamp_f64_to_f32(ratio.clamp(0.0, 1.0))
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
    if miles == 0.0 && state.ledger.current_day_miles == 0.0 {
        effective_kind = TravelDayKind::NonTravel;
    }
    // A stop cannot erase driving already completed earlier in the same day.
    if miles == 0.0 && state.ledger.current_day_miles > 0.0 {
        effective_kind = state
            .ledger
            .current_day_kind
            .unwrap_or(TravelDayKind::Partial);
    }

    match state.ledger.current_day_kind {
        None => {
            apply_initial_counters(state, effective_kind);
            state.ledger.current_day_kind = Some(effective_kind);
        }
        Some(existing) if existing != effective_kind => {
            adjust_counters_for_transition(state, existing, effective_kind);
            state.ledger.current_day_kind = Some(effective_kind);
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
            state.ledger.current_day_miles =
                (state.miles_traveled_actual - state.prev_miles_traveled).max(0.0);
            miles = credited;
        } else {
            miles = 0.0;
        }
    }

    match effective_kind {
        TravelDayKind::Travel => {
            state.day_state.travel.traveled_today = true;
            state.day_state.travel.partial_traveled_today = false;
        }
        TravelDayKind::Partial => {
            state.day_state.travel.traveled_today = false;
            state.day_state.travel.partial_traveled_today = true;
        }
        TravelDayKind::NonTravel => {
            state.day_state.travel.traveled_today = false;
            state.day_state.travel.partial_traveled_today = false;
        }
    }

    if miles > 0.0
        && state.endgame.active
        && matches!(effective_kind, TravelDayKind::Partial)
        && state.endgame.wear_shave_ratio < 1.0
    {
        apply_endgame_wear_shave(state);
    }

    (effective_kind, miles)
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
        (TravelDayKind::NonTravel, TravelDayKind::Travel | TravelDayKind::Partial) => {
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
    use crate::constants::TRAVEL_HISTORY_WINDOW;
    use crate::state::{GameMode, PolicyKind};
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
    fn late_route_camping_never_moves_the_van_or_finishes_the_route() {
        for forage in [false, true] {
            let mut state = fresh_state();
            state.trail_distance = 2_400.0;
            state.miles_traveled_actual = 2_398.0;
            state.miles_traveled = state.miles_traveled_actual;
            state.endgame.active = true;
            state.endgame.stop_cap_max_full = 0;
            state.stats.supplies = 5;
            state.stats.sanity = 3;
            let before = state.clone();
            let config = crate::camp::CampConfig::default_config();
            if forage {
                crate::camp::camp_forage(&mut state, &config);
            } else {
                crate::camp::camp_rest(&mut state, &config);
            }
            assert!(
                (state.miles_traveled_actual - before.miles_traveled_actual).abs() < f32::EPSILON
            );
            assert!(!state.boss.readiness.ready);
            if forage {
                assert_eq!(state.day, before.day);
                assert_eq!(
                    state.continuity.clock_minutes,
                    before.continuity.clock_minutes + 120
                );
                assert!(state.day_records.is_empty());
            } else {
                assert_eq!(state.day, before.day + 1);
                let day = state.day_records.last().unwrap();
                assert_eq!(day.kind, TravelDayKind::NonTravel);
                assert!(day.miles.abs() < f32::EPSILON);
            }
        }
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
        assert!(state.day_state.travel.traveled_today);
        assert!(!state.day_state.travel.partial_traveled_today);
        state.end_of_day();
        assert_eq!(state.travel_days, 1);
        assert_eq!(state.partial_travel_days, 1);
        assert_eq!(state.non_travel_days, 1);
    }

    #[test]
    fn recent_stops_cannot_create_distance() {
        let mut state = fresh_state();
        state.recent_travel_days =
            VecDeque::from(vec![TravelDayKind::NonTravel; TRAVEL_HISTORY_WINDOW]);
        let (kind, miles) = record_travel_day(&mut state, TravelDayKind::NonTravel, 0.0);
        assert_eq!(kind, TravelDayKind::NonTravel);
        assert_eq!((miles).to_bits(), (0.0f32).to_bits());
        assert_eq!((state.miles_traveled_actual).to_bits(), (0.0f32).to_bits());

        state.mode = GameMode::Deep;
        state.policy = Some(PolicyKind::Conservative);
        state.recent_travel_days =
            VecDeque::from(vec![TravelDayKind::NonTravel; TRAVEL_HISTORY_WINDOW]);
        let (kind, miles) = record_travel_day(&mut state, TravelDayKind::NonTravel, 0.0);
        assert_eq!(kind, TravelDayKind::NonTravel);
        assert_eq!((miles).to_bits(), (0.0f32).to_bits());
        assert_eq!((state.miles_traveled_actual).to_bits(), (0.0f32).to_bits());
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
    fn fractional_leg_credit_matches_observed_progress() {
        let mut state = fresh_state();
        record_travel_day(&mut state, TravelDayKind::Travel, 1_498.3);
        for requested in [0.17, 11.3, 0.1, 7.127] {
            let before = state.miles_traveled_actual;
            let (_, credited) = record_travel_day(&mut state, TravelDayKind::Partial, requested);
            let observed = state.miles_traveled_actual - before;
            assert_eq!(credited.to_bits(), observed.to_bits());
        }
    }

    #[test]
    fn fractional_days_reconcile_with_odometer_and_saved_open_day() {
        let mut state = fresh_state();
        state.trail_distance = 2_100.0;
        'journey: for _ in 0..100 {
            state.start_of_day();
            let before = state.miles_traveled_actual;
            for leg in [0.17, 27.123, 0.1, 11.707, 6.39] {
                state.record_travel_day(TravelDayKind::Travel, leg, "driving");
                assert_eq!(
                    state.ledger.current_day_miles.to_bits(),
                    (state.miles_traveled_actual - before).to_bits()
                );
                if state.miles_traveled_actual >= state.trail_distance {
                    break 'journey;
                }
            }
            state.end_of_day();
        }
        assert_eq!(state.miles_traveled_actual.to_bits(), 2_100.0_f32.to_bits());
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        for mut checkpoint in [state, restored] {
            let records: Vec<_> = checkpoint
                .day_records
                .iter()
                .chain(checkpoint.ledger.current_day_record.iter())
                .cloned()
                .collect();
            let open = compute_day_ledger_metrics(&records);
            assert_eq!(
                open.total_miles.to_bits(),
                checkpoint.miles_traveled_actual.to_bits()
            );
            checkpoint.end_of_day();
            let closed = checkpoint.ledger_metrics();
            assert_eq!(closed.total_miles.to_bits(), 2_100.0_f32.to_bits());
            assert_eq!(closed.total_days, open.total_days);
        }
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
            let kind = match rng.gen_range(0..3) {
                0 => TravelDayKind::Travel,
                1 => TravelDayKind::Partial,
                _ => TravelDayKind::NonTravel,
            };
            let miles = match kind {
                TravelDayKind::Travel => rng.gen_range(5.0..18.0),
                TravelDayKind::Partial => rng.gen_range(2.0..9.0),
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
        let manual_ratio = compute_ratio(travel + partial, total);
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
