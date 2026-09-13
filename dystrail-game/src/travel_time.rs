//! Road distance and hazards use the same clock as repairs, gathering and town work.
use crate::GameState;

pub const TRAVEL_DAY_START: u16 = 8 * 60;
pub const TRAVEL_DAY_END: u16 = 13 * 60;
pub const TRAVEL_DAY_MINUTES: u16 = TRAVEL_DAY_END - TRAVEL_DAY_START;
pub const TRAVEL_LEG_MINUTES: u16 = 60;
/// The configured pace penalty is earned over five actual driving hours.
const PACE_FATIGUE_MINUTES: u64 = 300;

/// Convert a full driving day's probability to the exposure in one travel leg.
#[must_use]
pub fn probability_for_minutes(daily: f32, minutes: u16) -> f32 {
    1.0 - (1.0 - daily.clamp(0.0, 1.0)).powf(f32::from(minutes) / f32::from(TRAVEL_DAY_MINUTES))
}

impl GameState {
    #[must_use]
    pub fn travel_minutes_available(&self) -> u16 {
        TRAVEL_DAY_END.saturating_sub(self.continuity.clock_minutes.max(TRAVEL_DAY_START))
    }

    #[must_use]
    pub fn travel_leg_minutes(&self) -> u16 {
        self.travel_minutes_available().min(TRAVEL_LEG_MINUTES)
    }

    pub(crate) fn prepare_travel_clock(&mut self) {
        if self.continuity.clock_minutes >= TRAVEL_DAY_END {
            self.start_of_day();
            self.end_of_day();
        }
        self.continuity.clock_minutes = self.continuity.clock_minutes.max(TRAVEL_DAY_START);
    }

    /// Distance has already been recorded; spending its time must never credit it again.
    pub(crate) fn spend_driving_time(&mut self, minutes: u16) {
        self.continuity.clock_minutes = self.continuity.clock_minutes.saturating_add(minutes);
        self.continuity.driving_minutes_total = self
            .continuity
            .driving_minutes_total
            .saturating_add(u32::from(minutes));
        self.accrue_pace_fatigue(minutes);
    }

    /// Whole costs belong to the drive that earns them; fractional debt never expires overnight.
    fn accrue_pace_fatigue(&mut self, minutes: u16) {
        if minutes == 0 {
            return;
        }
        let pacing = crate::pacing::PacingConfig::default_config();
        let pace = pacing.get_pace_safe(self.pace.as_str());
        if pace.sanity >= 0 {
            return;
        }
        let units = u64::from(self.continuity.pace_fatigue_remainder)
            + u64::from(pace.sanity.unsigned_abs()) * u64::from(minutes);
        let loss = i32::try_from(units / PACE_FATIGUE_MINUTES).unwrap_or(i32::MAX);
        self.continuity.pace_fatigue_remainder =
            u16::try_from(units % PACE_FATIGUE_MINUTES).unwrap_or(0);
        self.stats.sanity = self.stats.sanity.saturating_sub(loss).max(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pace_fatigue_full_day_cost_is_independent_of_driving_chunk_size() {
        for (pace, loss) in [
            (crate::PaceId::Steady, 0),
            (crate::PaceId::Heated, 1),
            (crate::PaceId::Blitz, 2),
        ] {
            for chunks in [vec![300], vec![60; 5], vec![1; 300], vec![59, 1, 90, 150]] {
                let mut gs = GameState {
                    pace,
                    ..GameState::default()
                };
                gs.stats.sanity = 8;
                for minutes in chunks {
                    gs.spend_driving_time(minutes);
                }
                assert_eq!(gs.stats.sanity, 8 - loss, "{pace:?}");
                assert_eq!(gs.continuity.pace_fatigue_remainder, 0);
                assert_eq!(gs.continuity.driving_minutes_total, 300);
                assert_eq!(gs.continuity.clock_minutes, TRAVEL_DAY_END);
                assert_eq!(gs.miles_traveled_actual.to_bits(), 0.0_f32.to_bits());
            }
        }
    }

    #[test]
    fn pace_fatigue_partial_minutes_survive_pace_changes_and_zero_time() {
        let mut gs = GameState::default();
        gs.stats.sanity = 8;
        gs.pace = crate::PaceId::Heated;
        gs.spend_driving_time(149);
        assert_eq!(gs.continuity.pace_fatigue_remainder, 149);
        gs.pace = crate::PaceId::Steady;
        gs.spend_driving_time(60);
        assert_eq!(gs.continuity.pace_fatigue_remainder, 149);
        gs.pace = crate::PaceId::Blitz;
        gs.spend_driving_time(75);
        assert_eq!(gs.continuity.pace_fatigue_remainder, 299);
        assert_eq!(gs.stats.sanity, 8);
        let before_zero = serde_json::to_value(&gs).unwrap();
        gs.spend_driving_time(0);
        assert_eq!(serde_json::to_value(&gs).unwrap(), before_zero);
        gs.pace = crate::PaceId::Heated;
        gs.spend_driving_time(1);
        assert_eq!(gs.stats.sanity, 7);
        assert_eq!(gs.continuity.pace_fatigue_remainder, 0);
        assert_eq!(gs.continuity.driving_minutes_total, 285);
    }

    #[test]
    fn pace_fatigue_remainder_survives_reload_and_older_saves_default_to_zero() {
        let mut gs = GameState {
            pace: crate::PaceId::Heated,
            ..GameState::default()
        };
        gs.stats.sanity = 8;
        gs.spend_driving_time(137);
        let saved = serde_json::to_string(&gs).unwrap();
        let mut restored: GameState = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored.continuity.pace_fatigue_remainder, 137);
        gs.spend_driving_time(163);
        restored.spend_driving_time(163);
        assert_eq!(restored.stats, gs.stats);
        assert_eq!(restored.stats.sanity, 7);
        assert_eq!(restored.continuity.pace_fatigue_remainder, 0);
        assert_eq!(restored.continuity.driving_minutes_total, 300);
        assert_eq!(
            restored.continuity.clock_minutes,
            gs.continuity.clock_minutes
        );

        let mut legacy: serde_json::Value = serde_json::from_str(&saved).unwrap();
        assert!(
            legacy
                .as_object_mut()
                .unwrap()
                .remove("pace_fatigue_remainder")
                .is_some()
        );
        let legacy: GameState = serde_json::from_value(legacy).unwrap();
        assert_eq!(legacy.continuity.pace_fatigue_remainder, 0);
        assert_eq!(legacy.stats.sanity, 8);
        assert_eq!(legacy.continuity.driving_minutes_total, 137);
    }

    #[test]
    fn pace_fatigue_does_not_replace_daily_diet_or_generic_daily_effects() {
        for (diet, diet_sanity) in [
            (crate::DietId::Quiet, 3),
            (crate::DietId::Mixed, 2),
            (crate::DietId::Doom, -2),
        ] {
            for pace in [
                crate::PaceId::Steady,
                crate::PaceId::Heated,
                crate::PaceId::Blitz,
            ] {
                let mut gs = GameState {
                    pace,
                    diet,
                    ..GameState::default()
                };
                gs.stats.sanity = 4;
                gs.weather_state.neutral_buffer = 100;
                gs.journey_daily.sanity.base = 1.0;
                gs.continuity.pace_fatigue_remainder = 299;
                gs.start_of_day();
                assert_eq!(gs.stats.sanity, 4 - 1 + diet_sanity, "{pace:?} {diet:?}");
                assert_eq!(gs.continuity.pace_fatigue_remainder, 299);
                assert_eq!(gs.continuity.driving_minutes_total, 0);
                let stats = gs.stats.clone();
                gs.start_of_day();
                assert_eq!(gs.stats, stats, "daily effects repeated");
            }
        }
    }

    #[test]
    fn pace_fatigue_is_not_earned_by_work_camp_or_menu_time() {
        let (_, mut gs) = clear_day();
        gs.pace = crate::PaceId::Blitz;
        gs.stats.sanity = 4;
        gs.continuity.pace_fatigue_remainder = 299;
        gs.continuity.route_services.stop = Some(160);
        let before = gs.clone();
        assert!(gs.perform_activity(crate::activities::Activity::WorkCash));
        assert_eq!(
            gs.stats.sanity, 3,
            "only the paid shift's stated cost applies"
        );
        assert_eq!(
            gs.continuity.clock_minutes,
            before.continuity.clock_minutes + 180
        );
        assert_eq!(gs.continuity.driving_minutes_total, 0);
        assert_eq!(gs.continuity.pace_fatigue_remainder, 299);

        let before_menu = gs.clone();
        gs.advance_clock(&before_menu, 0);
        assert_eq!(
            serde_json::to_value(&gs).unwrap(),
            serde_json::to_value(&before_menu).unwrap()
        );
        let camp = crate::camp::CampConfig::default_config();
        assert!(crate::camp::camp_rest(&mut gs, &camp).rested);
        assert_eq!(gs.day, before.day + camp.rest.day);
        assert_eq!(gs.stats.sanity, (3 + camp.rest.sanity).min(10));
        assert_eq!(gs.continuity.clock_minutes, TRAVEL_DAY_START);
        assert_eq!(gs.continuity.driving_minutes_total, 0);
        assert_eq!(gs.continuity.pace_fatigue_remainder, 299);
        assert_eq!(
            gs.miles_traveled_actual.to_bits(),
            before.miles_traveled_actual.to_bits()
        );
        gs.start_of_day();
        assert_eq!(
            gs.continuity.pace_fatigue_remainder, 299,
            "overnight erased earned fatigue"
        );
    }

    #[test]
    fn pace_fatigue_on_a_short_drive_is_part_of_its_stats_and_not_weather_cost() {
        let (mut controller, mut gs) = clear_day();
        gs.pace = crate::PaceId::Heated;
        gs.stats.sanity = 8;
        gs.continuity.pace_fatigue_remainder = 290;
        gs.continuity.clock_minutes = TRAVEL_DAY_END - 10;
        let before = gs.clone();
        let outcome = controller.tick_day(&mut gs);
        assert!(!outcome.ended);
        assert_eq!(gs.continuity.driving_minutes_total, 10);
        assert_eq!(gs.stats.sanity - before.stats.sanity, -1);
        assert_eq!(gs.continuity.pace_fatigue_remainder, 0);
        assert_eq!(
            gs.continuity.weather_impact,
            before.continuity.weather_impact
        );
        assert_eq!(gs.day, before.day + 1);
        assert_eq!(gs.continuity.clock_minutes, TRAVEL_DAY_START);
        assert!((gs.miles_traveled_actual - 70.0 / 6.0).abs() < 0.001);
    }

    #[test]
    fn pace_fatigue_keeps_sanity_nonnegative() {
        let mut gs = GameState {
            pace: crate::PaceId::Blitz,
            ..GameState::default()
        };
        gs.stats.sanity = 1;
        gs.spend_driving_time(300);
        assert_eq!(gs.stats.sanity, 0);
        assert_eq!(gs.continuity.pace_fatigue_remainder, 0);
    }

    fn clear_day() -> (crate::JourneyController, GameState) {
        let mut cfg = crate::journey::JourneyCfg::default();
        cfg.breakdown.base = 0.0;
        cfg.wear.base = 0.0;
        let controller = crate::JourneyController::with_config(
            crate::MechanicalPolicyId::DystrailLegacy,
            crate::PolicyId::Classic,
            crate::StrategyId::Balanced,
            cfg,
            42,
            crate::EndgameTravelCfg::default_config(),
        );
        let mut gs = GameState::default().with_seed(
            42,
            crate::GameMode::Classic,
            crate::EncounterData::empty(),
        );
        gs.continuity.route_services.route_id = Some("uninterrupted-test-road".into());
        controller.configure_state(&mut gs);
        gs.start_of_day();
        gs.weather_state.today = crate::Weather::Clear;
        gs.weather_travel_multiplier = 1.0;
        gs.exec_travel_multiplier = 1.0;
        gs.exec_breakdown_bonus = 0.0;
        gs.illness_travel_penalty = 1.0;
        gs.stats.hp = 10;
        gs.stats.sanity = 10;
        (controller, gs)
    }

    fn drive_until_tomorrow(controller: &mut crate::JourneyController, gs: &mut GameState) -> f32 {
        let day = gs.day;
        let start = gs.miles_traveled_actual;
        for _ in 0..20 {
            if gs.day > day {
                break;
            }
            let outcome = controller.tick_day(gs);
            assert!(!outcome.ended);
        }
        assert_eq!(gs.day, day + 1);
        gs.miles_traveled_actual - start
    }

    #[test]
    fn each_hour_of_foraging_reduces_distance_by_one_hour_at_the_actual_speed() {
        for (pace, mph) in [
            (crate::PaceId::Steady, 60.0f32),
            (crate::PaceId::Heated, 70.0),
            (crate::PaceId::Blitz, 80.0),
        ] {
            let (mut controller, mut direct) = clear_day();
            direct.pace = pace;
            let mut stopped = direct.clone();
            assert!(stopped.perform_activity(crate::activities::Activity::Forage));
            let direct_miles = drive_until_tomorrow(&mut controller, &mut direct);
            let (mut stopped_controller, _) = clear_day();
            let stopped_miles = drive_until_tomorrow(&mut stopped_controller, &mut stopped);
            assert!(mph.mul_add(-5.0, direct_miles).abs() < 0.05);
            assert!(mph.mul_add(-3.0, stopped_miles).abs() < 0.05);
            assert!(mph.mul_add(-2.0, direct_miles - stopped_miles).abs() < 0.05);
            assert_eq!(direct.day_records.len(), 1);
            assert_eq!(stopped.day_records.len(), 1);
            assert_eq!(direct.continuity.driving_minutes_total, 300);
            assert_eq!(stopped.continuity.driving_minutes_total, 180);
        }
    }

    #[test]
    fn stops_use_the_driving_budget_and_reload_cannot_restore_it() {
        let mut gs = GameState::default();
        let before = gs.clone();
        gs.advance_clock(&before, 120);
        assert_eq!(gs.travel_minutes_available(), 180);
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
        assert_eq!(restored.travel_minutes_available(), 180);
        assert_eq!(restored.continuity.driving_minutes_total, 0);
        assert_eq!(
            restored.ledger.current_day_record,
            gs.ledger.current_day_record
        );
        let record = restored.ledger.current_day_record.as_ref().unwrap();
        assert_eq!(record.day_index, 0);
        assert_eq!(record.kind, crate::TravelDayKind::NonTravel);
        assert_eq!(record.miles.to_bits(), 0.0_f32.to_bits());
        assert!(!restored.day_state.lifecycle.day_initialized);
    }

    #[test]
    fn unfinished_work_carries_into_tomorrow_and_daily_costs_apply_once() {
        let (mut controller, mut gs) = clear_day();
        let _ = controller.tick_day(&mut gs);
        let traveled = gs.miles_traveled_actual;
        gs.continuity.clock_minutes = TRAVEL_DAY_END - 60;
        let before = gs.clone();
        gs.advance_clock(&before, 240);
        assert_eq!(gs.day, 2);
        assert_eq!(gs.continuity.clock_minutes, TRAVEL_DAY_START + 180);
        assert_eq!(gs.day_records.len(), 1);
        assert!((gs.day_records[0].miles - traveled).abs() < 0.001);
        assert!(!gs.day_state.lifecycle.day_initialized);
        let pending_day = gs.ledger.current_day_record.clone().unwrap();
        assert_eq!(pending_day.day_index, 1);
        assert_eq!(pending_day.kind, crate::TravelDayKind::NonTravel);
        assert_eq!(pending_day.miles.to_bits(), 0.0_f32.to_bits());
        gs.start_of_day();
        assert_eq!(gs.ledger.current_day_record.as_ref(), Some(&pending_day));
        let stats = gs.stats.clone();
        let before = gs.clone();
        gs.advance_clock(&before, 30);
        gs.start_of_day();
        assert_eq!(gs.stats, stats);
        assert_eq!(gs.day_records.len(), 1);
    }

    #[test]
    fn stationary_clock_chunks_record_only_days_with_elapsed_time() {
        for (clock, minutes, elapsed_days, final_clock, completed_days) in [
            (TRAVEL_DAY_START, 0, 0, TRAVEL_DAY_START, 0),
            (TRAVEL_DAY_START, 120, 0, TRAVEL_DAY_START + 120, 0),
            (TRAVEL_DAY_END - 120, 120, 0, TRAVEL_DAY_END, 0),
            (TRAVEL_DAY_END - 60, 120, 1, TRAVEL_DAY_START + 60, 1),
            (TRAVEL_DAY_END, 120, 1, TRAVEL_DAY_START + 120, 1),
        ] {
            let mut gs = GameState::default();
            gs.continuity.clock_minutes = clock;
            let before = gs.clone();
            gs.advance_clock(&before, minutes);
            assert_eq!(gs.day, before.day + elapsed_days);
            assert_eq!(gs.continuity.clock_minutes, final_clock);
            assert_eq!(gs.day_records.len(), completed_days);
            let active_minutes = elapsed_days * u32::from(TRAVEL_DAY_MINUTES)
                + u32::from(final_clock)
                - u32::from(clock);
            assert_eq!(active_minutes, u32::from(minutes));
            assert_eq!(gs.continuity.driving_minutes_total, 0);
            assert_eq!(
                gs.miles_traveled_actual.to_bits(),
                before.miles_traveled_actual.to_bits()
            );
            assert!(!gs.day_state.lifecycle.day_initialized);
            if minutes == 0 {
                assert!(gs.ledger.current_day_record.is_none());
                assert_eq!(
                    serde_json::to_value(&gs).unwrap(),
                    serde_json::to_value(&before).unwrap()
                );
            } else {
                let record = gs.ledger.current_day_record.as_ref().unwrap();
                assert_eq!(u32::from(record.day_index), gs.day - 1);
                assert_eq!(record.kind, crate::TravelDayKind::NonTravel);
                assert_eq!(record.miles.to_bits(), 0.0_f32.to_bits());
                assert!(gs.day_records.iter().all(|day| {
                    day.kind == crate::TravelDayKind::NonTravel
                        && day.miles.to_bits() == 0.0_f32.to_bits()
                }));
            }
        }
    }

    #[test]
    fn rest_preserves_miles_already_driven_and_never_credits_stationary_days() {
        let (mut controller, mut gs) = clear_day();
        let _ = controller.tick_day(&mut gs);
        let miles = gs.miles_traveled_actual;
        let outcome = crate::camp::camp_rest(&mut gs, &crate::camp::CampConfig::default_config());
        assert!(outcome.rested);
        assert_eq!(gs.day_records.len(), 1);
        assert!((gs.day_records[0].miles - miles).abs() < 0.001);
        assert!((gs.miles_traveled_actual - miles).abs() < 0.001);
        gs.advance_days(3);
        assert!((gs.miles_traveled_actual - miles).abs() < 0.001);
        assert!(gs.day_records[1..].iter().all(|record| record.miles == 0.0));
    }

    #[test]
    fn hourly_probabilities_compose_to_the_configured_daily_risk() {
        for daily in [0.01, 0.12, 0.5, 0.9] {
            let hourly = probability_for_minutes(daily, 60);
            let full_day = 1.0 - (1.0 - hourly).powi(i32::from(TRAVEL_DAY_MINUTES / 60));
            assert!((full_day - daily).abs() < 0.00001);
        }
        assert_eq!(probability_for_minutes(0.5, 0).to_bits(), 0.0f32.to_bits());
    }
}
