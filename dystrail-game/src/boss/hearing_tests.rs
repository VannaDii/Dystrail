use super::*;
use crate::{EncounterData, GameMode};

fn scripted(base: f64, sanity: i32, rolls: &[u32], guarantee: bool) -> HearingReport {
    let mut rolls = rolls.iter();
    let report = hearing::resolve(
        HearingReport {
            rules_version: hearing::HEARING_RULES_VERSION,
            starting_stats: crate::state::Stats {
                sanity,
                ..crate::state::Stats::default()
            },
            day: 12,
            minute: 660,
            base_chance: base,
            policy_guarantee: guarantee,
            rounds: vec![],
            adjusted_chance: None,
            vote_roll: None,
            outcome: HearingOutcome::Exhausted,
        },
        3,
        2,
        |kind| {
            let roll = *rolls.next().expect("unexpected random draw");
            assert!(roll < kind.bound());
            roll
        },
    );
    assert!(rolls.next().is_none(), "a scripted draw was not consumed");
    report
}

#[test]
fn one_two_and_three_rounds_use_only_completed_influences() {
    let one = scripted(0.55, 7, &[70, 50, 42], false);
    let two = scripted(0.55, 7, &[70, 49, 40, 50, 42], false);
    let three = scripted(0.55, 7, &[70, 49, 40, 49, 100, 42], false);
    for (report, count, average, sanity, chance) in [
        (one, 1, 120.0, 5, 0.66),
        (two, 2, 105.0, 3, 0.5775),
        (three, 3, 120.0, 1, 0.66),
    ] {
        assert_eq!(report.rounds.len(), count);
        assert!((report.average_through(count) - average).abs() < 1e-10);
        assert_eq!(report.rounds.last().unwrap().sanity_after, sanity);
        assert!((report.adjusted_chance.unwrap() - chance).abs() < 1e-10);
        assert_eq!(report.outcome, HearingOutcome::Passed);
    }
}

#[test]
fn vote_boundary_and_raw_auto_threshold_are_not_rounded() {
    assert_eq!(
        scripted(0.55, 7, &[70, 50, 65], false).outcome,
        HearingOutcome::Passed
    );
    assert_eq!(
        scripted(0.55, 7, &[70, 50, 66], false).outcome,
        HearingOutcome::Failed
    );
    let automatic = scripted(0.80, 7, &[90, 0, 80, 0, 70], false);
    assert!((automatic.adjusted_chance.unwrap() - 1.04).abs() < 1e-10);
    assert_eq!(automatic.outcome, HearingOutcome::Secured);
    assert_eq!(automatic.vote_roll, None);
    let almost = scripted(0.6666, 7, &[100, 50, 99], false);
    assert!(almost.adjusted_chance.unwrap() < 1.0);
    assert_eq!(almost.vote_roll, Some(99));
}

#[test]
fn exhaustion_precedes_continuation_and_automatic_victory() {
    for (sanity, rolls, rounds) in [
        (0, vec![], 0),
        (2, vec![100], 1),
        (4, vec![100, 0, 100], 2),
        (5, vec![100, 0, 100, 0, 100], 3),
    ] {
        let report = scripted(1.0, sanity, &rolls, true);
        assert_eq!(report.outcome, HearingOutcome::Exhausted);
        assert_eq!(report.rounds.len(), rounds);
        assert_eq!(report.adjusted_chance, None);
        assert_eq!(report.vote_roll, None);
        if let Some(round) = report.rounds.last() {
            assert_eq!(round.sanity_after, 0);
            assert_eq!(round.continuation_roll, None);
        }
    }
    // A provisional 120% chance does not protect against the next cost or weak roll.
    let report = scripted(0.80, 7, &[100, 0, 0, 50, 70], false);
    assert_eq!(report.outcome, HearingOutcome::Passed);
    assert!((report.adjusted_chance.unwrap() - 0.8).abs() < 1e-10);
}

#[test]
fn guarantee_survives_weak_influence_but_never_exhaustion() {
    let report = scripted(1.0, 7, &[0, 0, 0, 0, 0], true);
    assert_eq!(report.outcome, HearingOutcome::Secured);
    assert_eq!(report.adjusted_chance, Some(1.0));
    assert_eq!(report.vote_roll, None);
}

#[test]
fn forecasting_is_read_only_and_accounts_for_conditional_rounds() {
    let cfg = BossConfig::default();
    for (sanity, chance) in [
        (0, 0.0),
        (1, 0.0),
        (2, 0.0),
        (3, 0.5),
        (4, 0.5),
        (5, 0.75),
        (6, 0.75),
        (7, 1.0),
        (10, 1.0),
    ] {
        let mut state =
            GameState::default().with_seed(42, GameMode::Classic, EncounterData::empty());
        state.stats.sanity = sanity;
        let before = serde_json::to_value(&state).unwrap();
        let forecast = hearing_forecast(&state, &cfg);
        assert!((forecast.survival_chance - chance).abs() < 1e-10);
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
}

#[test]
fn reveal_save_recovery_skip_and_repeat_never_change_the_committed_result() {
    let mut original =
        GameState::default().with_seed(19, GameMode::Classic, EncounterData::empty());
    original.stats.sanity = 10;
    let checkpoint = serde_json::to_string(&original).unwrap();
    let result = run_boss_minigame(&mut original, &BossConfig::default());
    let mut replay: GameState = serde_json::from_str(&checkpoint).unwrap();
    replay = replay.rehydrate(EncounterData::empty());
    assert_eq!(
        run_boss_minigame(&mut replay, &BossConfig::default()),
        result
    );
    assert_eq!(replay.boss.hearing, original.boss.hearing);
    let report = original.boss.hearing.clone().unwrap();
    let final_stats = original.stats.clone();
    let rng = serde_json::to_value(original.rng_bundle.as_deref().unwrap()).unwrap();
    for _ in 0..20 {
        let before_phase = original.boss.presentation;
        let count = report.revealed_rounds(before_phase);
        assert!(count <= report.rounds.len());
        original = serde_json::from_str::<GameState>(&serde_json::to_string(&original).unwrap())
            .unwrap()
            .rehydrate(EncounterData::empty());
        assert_eq!(
            run_boss_minigame(&mut original, &BossConfig::default()),
            result
        );
        assert_eq!(original.stats, final_stats);
        assert_eq!(original.boss.hearing.as_ref(), Some(&report));
        assert_eq!(
            serde_json::to_value(original.rng_bundle.as_deref().unwrap()).unwrap(),
            rng
        );
        if before_phase == HearingPhase::Complete {
            break;
        }
        original.boss.presentation = report.next_phase(before_phase);
    }
    assert_eq!(original.boss.presentation, HearingPhase::Complete);
    original.boss.presentation = HearingPhase::Verdict;
    assert_eq!(
        run_boss_minigame(&mut original, &BossConfig::default()),
        result
    );
    assert_eq!(original.stats, final_stats);
}

#[test]
fn legacy_attempts_are_preserved_without_manufacturing_a_new_report() {
    let legacy = r#"{"ready":true,"reached":true,"attempted":true,"victory":true}"#;
    let mut state = GameState {
        boss: serde_json::from_str(legacy).unwrap(),
        ..GameState::default()
    };
    let before = state.stats.clone();
    assert_eq!(
        run_boss_minigame(&mut state, &BossConfig::default()),
        BossOutcome::PassedCloture
    );
    assert!(state.boss.hearing.is_none());
    assert_eq!(state.stats, before);
}

#[test]
fn round_distribution_and_win_rates_match_the_exact_model() {
    // Convolve integer influence values, then weight hearing lengths 1/2/3 by 1/2,1/4,1/4.
    let mut distribution = vec![1_u64];
    let bases = [0.25, 0.55, 0.75, 0.80, 0.88];
    let expected = [0.2541, 0.5548, 0.7472, 0.7875, 0.8421];
    let mut rates = [0.0; 5];
    for count in 1..=3 {
        let mut next = vec![0_u64; distribution.len() + 150];
        for (sum, ways) in distribution.iter().enumerate() {
            for influence in 50..=150 {
                next[sum + influence] += ways;
            }
        }
        distribution = next;
        let weight = if count == 1 { 0.5 } else { 0.25 };
        let total = 101_f64.powi(count);
        for (sum, ways) in distribution.iter().enumerate() {
            for (i, base) in bases.iter().enumerate() {
                let threshold = base * f64::from(u32::try_from(sum).unwrap()) / f64::from(count);
                let passes = (0..100).filter(|roll| f64::from(*roll) < threshold).count();
                rates[i] += weight * f64::from(u32::try_from(*ways).unwrap()) / total
                    * f64::from(u32::try_from(passes).unwrap())
                    / 100.0;
            }
        }
    }
    for (rate, expected) in rates.into_iter().zip(expected) {
        assert!((rate - expected).abs() < 0.0002, "{rate} vs {expected}");
    }
}

#[test]
fn terminal_hearing_time_never_triggers_post_verdict_recovery_or_hazards() {
    for (minute, expected_minute, elapsed_days) in [
        (480, 600, 0),
        (660, 780, 0),
        (720, 540, 1),
        (780, 600, 1),
        (1110, 600, 1),
        (1410, 600, 1),
    ] {
        let mut state =
            GameState::default().with_seed(42, GameMode::Classic, EncounterData::empty());
        state.stats.sanity = 7;
        state.continuity.clock_minutes = minute;
        let before = state.clone();
        run_boss_minigame(&mut state, &BossConfig::default());
        let resolved = state.stats.clone();
        let rng = serde_json::to_value(state.rng_bundle.as_deref().unwrap()).unwrap();
        state.advance_clock(&before, 120);
        assert_eq!(state.stats, resolved);
        assert_eq!(
            serde_json::to_value(state.rng_bundle.as_deref().unwrap()).unwrap(),
            rng
        );
        assert_eq!(state.continuity.clock_minutes, expected_minute);
        assert_eq!(state.day, before.day + elapsed_days);
    }
}

#[test]
fn questioning_keeps_the_legacy_vote_stream_and_consumes_it_only_for_a_vote() {
    for seed in 0..100 {
        let mut state =
            GameState::default().with_seed(seed, GameMode::Classic, EncounterData::empty());
        state.stats.sanity = 10;
        let mut legacy =
            GameState::default().with_seed(seed, GameMode::Classic, EncounterData::empty());
        let expected = legacy.next_pct();
        run_boss_minigame(&mut state, &BossConfig::default());
        let report = state.boss.hearing.as_ref().unwrap();
        if let Some(roll) = report.vote_roll {
            assert_eq!(roll, expected);
        } else {
            assert_eq!(state.next_pct(), expected);
        }
    }
}

#[test]
fn policy_preparation_forecast_explains_costs_and_matches_committed_entry() {
    for (supplies, cents, expected_supplies, expected_cents, expected_sanity) in [
        (4, 2000, 4, 0, 3),
        (3, 2000, 0, 2000, 3),
        (3, 1999, 0, 0, 2),
    ] {
        let mut state = GameState::default().with_seed(42, GameMode::Deep, EncounterData::empty());
        state.policy = Some(PolicyKind::Aggressive);
        state.stats.sanity = 2;
        state.stats.supplies = supplies;
        state.budget_cents = cents;
        let before = serde_json::to_value(&state).unwrap();
        let forecast = hearing_forecast(&state, &BossConfig::default());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
        assert_eq!(forecast.entry_sanity, expected_sanity);
        assert_eq!(forecast.preparation_supplies, expected_supplies);
        assert_eq!(forecast.preparation_cents, expected_cents);
        assert!(
            (forecast.survival_chance - if expected_sanity == 3 { 0.5 } else { 0.0 }).abs() < 1e-10
        );
        run_boss_minigame(&mut state, &BossConfig::default());
        let report = state.boss.hearing.as_ref().unwrap();
        assert_eq!(report.starting_stats.sanity, forecast.entry_sanity);
        assert_eq!(report.starting_stats.supplies, supplies - expected_supplies);
        assert_eq!(state.budget_cents, cents - expected_cents);
        let resolved = serde_json::to_value(&state).unwrap();
        run_boss_minigame(&mut state, &BossConfig::default());
        assert_eq!(serde_json::to_value(&state).unwrap(), resolved);
    }
}
