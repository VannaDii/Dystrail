use anyhow::{Context, Result, ensure};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::TryFrom;

use crate::common::scenario::full_game::{
    full_game_aggressive_expectation, full_game_balanced_expectation,
    full_game_conservative_expectation, full_game_plan, full_game_resource_manager_expectation,
};
use crate::logic::seeds::SeedInfo;
use crate::logic::{GameTester, GameplayStrategy, PlayabilityMetrics};
use dystrail_game::GameMode;
use dystrail_game::OtDeluxe90sPolicy;
use dystrail_game::OtDeluxeCrossingMethod;
use dystrail_game::OtDeluxePace;
use dystrail_game::OtDeluxeRiver;
use dystrail_game::OtDeluxeRiverBed;
use dystrail_game::OtDeluxeTrailVariant;
use dystrail_game::otdeluxe_crossing_options;
use dystrail_game::otdeluxe_crossings::resolve_crossing;
use dystrail_game::otdeluxe_state::{OtDeluxeInventory, OtDeluxeRiverState};
use dystrail_game::otdeluxe_total_miles_for_variant;
use dystrail_game::state::CrossingOutcomeTelemetry;

#[derive(Debug, Clone)]
pub struct PlayabilityRecord {
    pub scenario_name: String,
    pub mode: GameMode,
    pub strategy: GameplayStrategy,
    pub seed_code: String,
    pub seed_value: u64,
    pub metrics: PlayabilityMetrics,
}

#[derive(Debug, Clone)]
pub struct PlayabilityAggregate {
    pub scenario_name: String,
    pub mode: GameMode,
    pub strategy: GameplayStrategy,
    pub iterations: usize,
    pub mean_days: f64,
    pub std_days: f64,
    pub mean_miles: f64,
    pub std_miles: f64,
    pub boss_reach_pct: f64,
    pub boss_win_pct: f64,
    pub pants_failure_pct: f64,
    pub mean_travel_ratio: f64,
    pub mean_unique_per_20: f64,
    pub mean_rotation_events: f64,
    pub pct_reached_2k_by_150: f64,
    pub min_unique_per_20: f64,
    pub min_travel_ratio: f64,
    pub mean_crossing_events: f64,
    pub crossing_permit_rate: f64,
    pub mean_crossing_bribes: f64,
    pub crossing_bribe_success_rate: f64,
    pub mean_crossing_detours: f64,
    pub crossing_failure_rate: f64,
}

fn ensure_f32_close(actual: f32, expected: f32, label: &str) -> Result<()> {
    ensure!(
        (actual - expected).abs() <= f32::EPSILON,
        "{label} expected {expected:.2}, got {actual:.2}"
    );
    Ok(())
}

fn validate_otdeluxe_parity_invariants() -> Result<()> {
    let policy = OtDeluxe90sPolicy::default();
    validate_otdeluxe_trail_geometry(&policy)?;
    validate_otdeluxe_travel_speed(&policy)?;
    validate_otdeluxe_action_costs(&policy)?;
    validate_otdeluxe_crossing_thresholds(&policy)?;
    validate_otdeluxe_crossing_options(&policy)?;
    validate_otdeluxe_ferry_wait_distribution(&policy)?;
    validate_otdeluxe_health(&policy)?;
    Ok(())
}

fn validate_otdeluxe_trail_geometry(policy: &OtDeluxe90sPolicy) -> Result<()> {
    let main = otdeluxe_total_miles_for_variant(&policy.trail, OtDeluxeTrailVariant::Main);
    let sublette =
        otdeluxe_total_miles_for_variant(&policy.trail, OtDeluxeTrailVariant::SubletteCutoff);
    let dalles =
        otdeluxe_total_miles_for_variant(&policy.trail, OtDeluxeTrailVariant::DallesShortcut);
    let both = otdeluxe_total_miles_for_variant(
        &policy.trail,
        OtDeluxeTrailVariant::SubletteAndDallesShortcut,
    );
    ensure!(main == 2083, "OTDeluxe main route miles {main} != 2083");
    ensure!(
        sublette == main.saturating_sub(94),
        "OTDeluxe Sublette cutoff miles {sublette} != main-94"
    );
    ensure!(
        dalles == main.saturating_sub(50),
        "OTDeluxe Dalles shortcut miles {dalles} != main-50"
    );
    ensure!(
        both == main.saturating_sub(144),
        "OTDeluxe Sublette+Dalles miles {both} != main-144"
    );
    Ok(())
}

fn validate_otdeluxe_travel_speed(policy: &OtDeluxe90sPolicy) -> Result<()> {
    ensure_f32_close(
        policy.travel.base_mpd_plains_steady_good,
        20.0,
        "OTDeluxe base MPD",
    )?;
    ensure_f32_close(
        policy.travel.terrain_mult_mountains,
        0.5,
        "OTDeluxe mountains multiplier",
    )?;
    ensure_f32_close(
        policy.travel.sick_member_speed_penalty,
        0.10,
        "OTDeluxe sick member speed penalty",
    )?;
    ensure_f32_close(policy.oxen.sick_ox_weight, 0.5, "OTDeluxe sick ox weight")?;
    ensure_f32_close(
        policy.oxen.min_for_base,
        4.0,
        "OTDeluxe min oxen for base speed",
    )?;
    ensure_f32_close(policy.oxen.min_to_move, 0.0, "OTDeluxe min oxen to move")?;
    validate_otdeluxe_mpd_examples(policy)?;
    Ok(())
}

fn validate_otdeluxe_mpd_examples(policy: &OtDeluxe90sPolicy) -> Result<()> {
    use dystrail_game::GameState;
    use dystrail_game::otdeluxe_state::OtDeluxePartyState;

    let mut base_state = GameState::default();
    base_state.ot_deluxe.oxen.healthy = 4;
    base_state.ot_deluxe.oxen.sick = 0;
    base_state.ot_deluxe.pace = OtDeluxePace::Steady;
    base_state.ot_deluxe.party = OtDeluxePartyState::from_names(["A", "B"]);
    base_state.ot_deluxe.travel.disease_speed_mult = 1.0;
    base_state.ot_deluxe.weather.snow_depth = 0.0;
    base_state.ot_deluxe.miles_traveled = 0.0;
    let miles = base_state.otdeluxe_miles_for_today(policy);
    ensure_f32_close(miles, 20.0, "OTDeluxe computed plains MPD")?;

    let mut mountain_state = base_state.clone();
    mountain_state.ot_deluxe.miles_traveled = 932.0;
    let miles = mountain_state.otdeluxe_miles_for_today(policy);
    ensure_f32_close(miles, 10.0, "OTDeluxe computed mountain MPD")?;

    let mut sick_state = base_state.clone();
    if let Some(member) = sick_state.ot_deluxe.party.members.get_mut(0) {
        member.sick_days_remaining = 1;
    }
    let miles = sick_state.otdeluxe_miles_for_today(policy);
    ensure_f32_close(miles, 18.0, "OTDeluxe sick member MPD")?;

    let mut sick_ox_state = base_state.clone();
    sick_ox_state.ot_deluxe.oxen.healthy = 3;
    sick_ox_state.ot_deluxe.oxen.sick = 1;
    let miles = sick_ox_state.otdeluxe_miles_for_today(policy);
    ensure_f32_close(miles, 17.5, "OTDeluxe sick ox MPD")?;

    Ok(())
}

fn validate_otdeluxe_action_costs(policy: &OtDeluxe90sPolicy) -> Result<()> {
    ensure!(
        policy.actions.rest_days_min == 1 && policy.actions.rest_days_max == 9,
        "OTDeluxe rest days range {min}..{max} != 1..9",
        min = policy.actions.rest_days_min,
        max = policy.actions.rest_days_max
    );
    ensure!(
        policy.actions.trade_cost_days == 1,
        "OTDeluxe trade cost days {days} != 1",
        days = policy.actions.trade_cost_days
    );
    ensure!(
        policy.actions.hunt_cost_days == 1,
        "OTDeluxe hunt cost days {days} != 1",
        days = policy.actions.hunt_cost_days
    );
    Ok(())
}

fn validate_otdeluxe_crossing_thresholds(policy: &OtDeluxe90sPolicy) -> Result<()> {
    ensure_f32_close(
        policy.crossings.ferry_min_depth_ft,
        2.5,
        "OTDeluxe ferry min depth",
    )?;
    ensure_f32_close(
        policy.crossings.float_min_depth_ft,
        1.5,
        "OTDeluxe float min depth",
    )?;
    ensure_f32_close(
        policy.crossings.wet_goods_min_depth_ft,
        2.5,
        "OTDeluxe wet goods min depth",
    )?;
    ensure_f32_close(
        policy.crossings.swamped_min_depth_ft,
        3.0,
        "OTDeluxe swamped min depth",
    )?;
    ensure!(
        policy.crossings.drying_cost_days == 1,
        "OTDeluxe drying days {days} != 1",
        days = policy.crossings.drying_cost_days
    );
    ensure!(
        policy.crossings.crossing_cost_days == 1,
        "OTDeluxe crossing cost days {days} != 1",
        days = policy.crossings.crossing_cost_days
    );
    ensure_f32_close(
        policy.crossings.guide_risk_mult,
        0.20,
        "OTDeluxe guide risk multiplier",
    )?;
    ensure!(
        policy.crossings.ferry_wait_days_min == 0 && policy.crossings.ferry_wait_days_max == 6,
        "OTDeluxe ferry wait days {min}..{max} != 0..6",
        min = policy.crossings.ferry_wait_days_min,
        max = policy.crossings.ferry_wait_days_max
    );
    Ok(())
}

fn validate_otdeluxe_health(policy: &OtDeluxe90sPolicy) -> Result<()> {
    use dystrail_game::otdeluxe_state::{OtDeluxeAfflictionKind, OtDeluxePartyState};
    ensure!(
        policy.health.recovery_baseline == -10,
        "OTDeluxe recovery baseline {baseline} != -10",
        baseline = policy.health.recovery_baseline
    );
    ensure!(
        policy.health.death_threshold == 140,
        "OTDeluxe death threshold {threshold} != 140",
        threshold = policy.health.death_threshold
    );
    ensure!(
        policy.health.label_ranges.good_max == 34
            && policy.health.label_ranges.fair_max == 69
            && policy.health.label_ranges.poor_max == 104
            && policy.health.label_ranges.very_poor_max == 139,
        "OTDeluxe health label ranges do not match parity defaults"
    );
    ensure!(
        policy.affliction.illness_duration_days == 10,
        "OTDeluxe illness duration {days} != 10",
        days = policy.affliction.illness_duration_days
    );
    ensure!(
        policy.affliction.injury_duration_days == 30,
        "OTDeluxe injury duration {days} != 30",
        days = policy.affliction.injury_duration_days
    );
    let mut party = OtDeluxePartyState::from_names(["A"]);
    let died = party.members[0].apply_affliction(OtDeluxeAfflictionKind::Illness, 3, None);
    ensure!(!died, "OTDeluxe first affliction should not be fatal");
    let died_on_repeat = party.members[0].apply_affliction(OtDeluxeAfflictionKind::Injury, 3, None);
    ensure!(died_on_repeat, "OTDeluxe repeat affliction should be fatal");
    Ok(())
}

fn validate_otdeluxe_crossing_options(policy: &OtDeluxe90sPolicy) -> Result<()> {
    let inventory = OtDeluxeInventory {
        cash_cents: policy.crossings.ferry_cost_cents,
        clothes_sets: policy.crossings.guide_cost_clothes_sets,
        ..OtDeluxeInventory::default()
    };
    let shallow = OtDeluxeRiverState {
        depth_ft: 1.4,
        width_ft: 200.0,
        swiftness: 0.4,
        bed: OtDeluxeRiverBed::Muddy,
    };
    let shallow_opts = otdeluxe_crossing_options(
        &policy.crossings,
        OtDeluxeRiver::Kansas,
        &shallow,
        &inventory,
    );
    ensure!(
        shallow_opts.ford() && !shallow_opts.caulk_float() && !shallow_opts.ferry(),
        "OTDeluxe crossing options allowed ferry/float below depth thresholds"
    );

    let mid = OtDeluxeRiverState {
        depth_ft: 2.4,
        width_ft: 200.0,
        swiftness: 0.4,
        bed: OtDeluxeRiverBed::Muddy,
    };
    let mid_opts =
        otdeluxe_crossing_options(&policy.crossings, OtDeluxeRiver::Kansas, &mid, &inventory);
    ensure!(
        mid_opts.caulk_float() && !mid_opts.ferry(),
        "OTDeluxe crossing options allowed ferry below min depth"
    );

    let deep = OtDeluxeRiverState {
        depth_ft: 2.6,
        width_ft: 200.0,
        swiftness: 0.4,
        bed: OtDeluxeRiverBed::Muddy,
    };
    let deep_opts =
        otdeluxe_crossing_options(&policy.crossings, OtDeluxeRiver::Kansas, &deep, &inventory);
    ensure!(
        deep_opts.caulk_float() && deep_opts.ferry(),
        "OTDeluxe crossing options missing ferry above depth threshold"
    );
    Ok(())
}

fn validate_otdeluxe_ferry_wait_distribution(policy: &OtDeluxe90sPolicy) -> Result<()> {
    let ferry_state = OtDeluxeRiverState {
        depth_ft: 3.0,
        width_ft: 200.0,
        swiftness: 0.4,
        bed: OtDeluxeRiverBed::Muddy,
    };
    let mut rng = SmallRng::seed_from_u64(42);
    let mut counts = [0u32; 7];
    let draws = 7000_u32;
    for _ in 0..draws {
        let resolution = resolve_crossing(
            &policy.crossings,
            OtDeluxeRiver::Kansas,
            &ferry_state,
            OtDeluxeCrossingMethod::Ferry,
            &mut rng,
        );
        let idx = usize::from(resolution.wait_days.min(6));
        counts[idx] = counts[idx].saturating_add(1);
    }
    let expected = f64::from(draws) / 7.0;
    let chi_square: f64 = counts
        .iter()
        .map(|&count| {
            let diff = f64::from(count) - expected;
            diff * diff / expected
        })
        .sum();
    ensure!(
        chi_square < 20.0,
        "OTDeluxe ferry wait distribution chi-square {chi_square:.2} exceeds threshold"
    );

    Ok(())
}

fn push_limited_warn(
    warn_counts: &mut BTreeMap<String, usize>,
    key: &str,
    limit: usize,
    message: impl FnOnce() -> String,
) {
    let counter = warn_counts.entry(key.to_string()).or_insert(0);
    if *counter < limit {
        println!("{}", message());
    }
    *counter += 1;
}

fn emit_record_warnings(record: &PlayabilityRecord, warn_counts: &mut BTreeMap<String, usize>) {
    warn_crossing_anomalies(record, warn_counts);
}

fn warn_crossing_anomalies(record: &PlayabilityRecord, warn_counts: &mut BTreeMap<String, usize>) {
    for (idx, event) in record.metrics.crossing_events.iter().enumerate() {
        if event.bribe_attempted && event.bribe_success.is_none() {
            push_limited_warn(
                warn_counts,
                &format!("{}::crossing{}_bribe_missing", record.scenario_name, idx),
                2,
                || {
                    format!(
                        "WARN: {} seed {} crossing {} missing bribe outcome despite attempt",
                        record.scenario_name, record.seed_code, idx
                    )
                },
            );
        }
        if !event.bribe_attempted && event.bribe_success.is_some() {
            push_limited_warn(
                warn_counts,
                &format!("{}::crossing{}_bribe_spurious", record.scenario_name, idx),
                2,
                || {
                    format!(
                        "WARN: {} seed {} crossing {} reported bribe success without attempt",
                        record.scenario_name, record.seed_code, idx
                    )
                },
            );
        }
        let is_detoured = matches!(event.outcome, CrossingOutcomeTelemetry::Detoured);
        if event.detour_taken && !is_detoured {
            push_limited_warn(
                warn_counts,
                &format!("{}::crossing{}_detour_mismatch", record.scenario_name, idx),
                2,
                || {
                    format!(
                        "WARN: {} seed {} crossing {} flagged detour but outcome {:?}",
                        record.scenario_name, record.seed_code, idx, event.outcome
                    )
                },
            );
        }
        if is_detoured && !event.detour_taken {
            push_limited_warn(
                warn_counts,
                &format!("{}::crossing{}_detour_missing", record.scenario_name, idx),
                2,
                || {
                    format!(
                        "WARN: {} seed {} crossing {} outcome detoured without detour flag",
                        record.scenario_name, record.seed_code, idx
                    )
                },
            );
        }
        if matches!(event.outcome, CrossingOutcomeTelemetry::Failed) && event.detour_taken {
            push_limited_warn(
                warn_counts,
                &format!("{}::crossing{}_failure_detour", record.scenario_name, idx),
                2,
                || {
                    format!(
                        "WARN: {} seed {} crossing {} recorded failure with detour flag",
                        record.scenario_name, record.seed_code, idx
                    )
                },
            );
        }
    }
}

const PLAYABILITY_SCENARIOS: &[(GameMode, GameplayStrategy)] = &[
    (GameMode::Classic, GameplayStrategy::Balanced),
    (GameMode::Classic, GameplayStrategy::Conservative),
    (GameMode::Classic, GameplayStrategy::Aggressive),
    (GameMode::Classic, GameplayStrategy::ResourceManager),
    (GameMode::Deep, GameplayStrategy::Balanced),
    (GameMode::Deep, GameplayStrategy::Conservative),
    (GameMode::Deep, GameplayStrategy::Aggressive),
    (GameMode::Deep, GameplayStrategy::ResourceManager),
];

pub fn run_playability_analysis(
    tester: &GameTester,
    seeds: &[SeedInfo],
    iterations: usize,
) -> Result<Vec<PlayabilityRecord>> {
    run_playability_analysis_with(tester, seeds, iterations, |mode, strategy| {
        add_expectations(full_game_plan(mode, strategy), strategy)
    })
}

fn run_playability_analysis_with<F>(
    tester: &GameTester,
    seeds: &[SeedInfo],
    iterations: usize,
    mut plan_builder: F,
) -> Result<Vec<PlayabilityRecord>>
where
    F: FnMut(GameMode, GameplayStrategy) -> crate::logic::SimulationPlan,
{
    let iterations = iterations.max(1);
    let mut records = Vec::with_capacity(seeds.len() * PLAYABILITY_SCENARIOS.len() * iterations);

    for &(mode, strategy) in PLAYABILITY_SCENARIOS {
        for seed in seeds.iter().filter(|seed| seed.matches_mode(mode)) {
            for iteration in 0..iterations {
                let iteration_offset = u64::try_from(iteration).unwrap_or(0);
                let iteration_seed = seed.seed.wrapping_add(iteration_offset);
                let plan = plan_builder(mode, strategy);
                let summary = tester.run_plan(&plan, iteration_seed);
                #[rustfmt::skip]
                let context = format!("Playability expectation failed for mode {:?}, strategy {}, seed {} (iteration {})", mode, strategy, seed.seed, iteration + 1);
                for expectation in &plan.expectations {
                    expectation
                        .evaluate(&summary)
                        .with_context(|| context.clone())?;
                }

                let metrics = summary.metrics.clone();
                let scenario_name = format!("{} - {}", mode_label(mode), strategy);
                let seed_code = dystrail_game::encode_friendly(mode.is_deep(), iteration_seed);

                #[rustfmt::skip]
                let record = PlayabilityRecord { scenario_name, mode, strategy, seed_code, seed_value: iteration_seed, metrics };
                records.push(record);
            }
        }
    }

    Ok(records)
}

pub fn aggregate_playability(records: &[PlayabilityRecord]) -> Vec<PlayabilityAggregate> {
    let mut aggregates: BTreeMap<String, AggregateBuilder> = BTreeMap::new();
    let mut warn_counts: BTreeMap<String, usize> = BTreeMap::new();

    for record in records {
        let entry = aggregates
            .entry(record.scenario_name.clone())
            .or_insert_with(|| AggregateBuilder::new(record));
        entry.ingest(&record.metrics);
        emit_record_warnings(record, &mut warn_counts);
    }

    aggregates
        .into_values()
        .map(AggregateBuilder::finish)
        .collect()
}

pub fn validate_playability_targets(
    _aggregates: &[PlayabilityAggregate],
    records: &[PlayabilityRecord],
) -> Result<()> {
    validate_otdeluxe_parity_invariants()?;
    validate_crossing_determinism(records)?;
    ensure_crossing_consistency(records)?;
    Ok(())
}

fn validate_crossing_determinism(records: &[PlayabilityRecord]) -> Result<()> {
    for record in records {
        let determinism_violation = record.metrics.crossing_events.iter().any(|event| {
            event.bribe_success == Some(true)
                && matches!(event.outcome, CrossingOutcomeTelemetry::Failed)
        });
        ensure!(
            !determinism_violation,
            "Crossing determinism violated for scenario {} seed {}",
            record.scenario_name,
            record.seed_code
        );
    }
    Ok(())
}

fn ensure_crossing_consistency(records: &[PlayabilityRecord]) -> Result<()> {
    let mut scenario_outcomes: BTreeMap<
        (String, u64),
        HashMap<usize, HashSet<CrossingOutcomeTelemetry>>,
    > = BTreeMap::new();

    for record in records {
        let entry = scenario_outcomes
            .entry((record.scenario_name.clone(), record.seed_value))
            .or_default();
        for (idx, event) in record.metrics.crossing_events.iter().enumerate() {
            entry.entry(idx).or_default().insert(event.outcome);
        }
    }

    for ((scenario, seed), outcome_map) in scenario_outcomes {
        for (idx, outcomes) in outcome_map {
            ensure!(
                outcomes.len() <= 1,
                "Crossing index {idx} in scenario {scenario} seed {seed} produced mixed outcomes {outcomes:?}",
            );
        }
    }

    Ok(())
}

const fn mode_label(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Classic => "Classic",
        GameMode::Deep => "Deep",
    }
}

#[derive(Debug, Clone)]
struct AggregateBuilder {
    scenario_name: String,
    mode: GameMode,
    strategy: GameplayStrategy,
    stats_days: RunningStats,
    stats_miles: RunningStats,
    iterations: u32,
    boss_reached: u32,
    boss_won: u32,
    pants_failures: u32,
    travel_ratio_sum: f64,
    unique_per_20_sum: f64,
    rotation_event_sum: u32,
    milestone_hits: u32,
    min_unique_per_20: f64,
    min_travel_ratio: f64,
    crossing_events_sum: u32,
    crossing_permit_sum: u32,
    crossing_bribe_attempt_sum: u32,
    crossing_bribe_success_sum: u32,
    crossing_detour_sum: u32,
    crossing_failure_sum: u32,
}

impl AggregateBuilder {
    fn new(record: &PlayabilityRecord) -> Self {
        Self {
            scenario_name: record.scenario_name.clone(),
            mode: record.mode,
            strategy: record.strategy,
            stats_days: RunningStats::default(),
            stats_miles: RunningStats::default(),
            iterations: 0,
            boss_reached: 0,
            boss_won: 0,
            pants_failures: 0,
            travel_ratio_sum: 0.0,
            unique_per_20_sum: 0.0,
            rotation_event_sum: 0,
            milestone_hits: 0,
            min_unique_per_20: f64::INFINITY,
            min_travel_ratio: f64::INFINITY,
            crossing_events_sum: 0,
            crossing_permit_sum: 0,
            crossing_bribe_attempt_sum: 0,
            crossing_bribe_success_sum: 0,
            crossing_detour_sum: 0,
            crossing_failure_sum: 0,
        }
    }

    fn ingest(&mut self, metrics: &PlayabilityMetrics) {
        self.iterations += 1;
        self.stats_days.add(f64::from(metrics.days_survived));
        self.stats_miles.add(f64::from(metrics.miles_traveled));
        if metrics.boss.reached {
            self.boss_reached += 1;
        }
        if metrics.boss.won {
            self.boss_won += 1;
        }
        if metrics.final_pants >= 100 || metrics.ending_type.contains("Pants") {
            self.pants_failures += 1;
        }
        if metrics.milestones.reached_2000_by_day150 {
            self.milestone_hits += 1;
        }
        self.travel_ratio_sum += metrics.travel_ratio;
        self.unique_per_20_sum += metrics.unique_per_20_days;
        self.rotation_event_sum = self
            .rotation_event_sum
            .saturating_add(metrics.rotation_events);
        self.min_unique_per_20 = self.min_unique_per_20.min(metrics.unique_per_20_days);
        self.min_travel_ratio = self.min_travel_ratio.min(metrics.travel_ratio);
        self.crossing_events_sum = self
            .crossing_events_sum
            .saturating_add(u32::try_from(metrics.crossing_events.len()).unwrap_or(0));
        self.crossing_permit_sum = self
            .crossing_permit_sum
            .saturating_add(metrics.crossing_permit_uses);
        self.crossing_bribe_attempt_sum = self
            .crossing_bribe_attempt_sum
            .saturating_add(metrics.crossing_bribe_attempts);
        self.crossing_bribe_success_sum = self
            .crossing_bribe_success_sum
            .saturating_add(metrics.crossing_bribe_successes);
        self.crossing_detour_sum = self
            .crossing_detour_sum
            .saturating_add(metrics.crossing_detours_taken);
        self.crossing_failure_sum = self
            .crossing_failure_sum
            .saturating_add(metrics.crossing_failures);
    }

    fn finish(self) -> PlayabilityAggregate {
        let iterations_u32 = self.iterations.max(1);
        let iterations = usize::try_from(self.iterations).unwrap_or(usize::MAX);
        let denom = f64::from(iterations_u32);
        PlayabilityAggregate {
            scenario_name: self.scenario_name,
            mode: self.mode,
            strategy: self.strategy,
            iterations,
            mean_days: self.stats_days.mean(),
            std_days: self.stats_days.std_dev(),
            mean_miles: self.stats_miles.mean(),
            std_miles: self.stats_miles.std_dev(),
            boss_reach_pct: f64::from(self.boss_reached) / denom,
            boss_win_pct: f64::from(self.boss_won) / denom,
            pants_failure_pct: f64::from(self.pants_failures) / denom,
            mean_travel_ratio: self.travel_ratio_sum / denom,
            mean_unique_per_20: self.unique_per_20_sum / denom,
            mean_rotation_events: f64::from(self.rotation_event_sum) / denom,
            pct_reached_2k_by_150: f64::from(self.milestone_hits) / denom,
            min_unique_per_20: if self.min_unique_per_20.is_finite() {
                self.min_unique_per_20
            } else {
                0.0
            },
            min_travel_ratio: if self.min_travel_ratio.is_finite() {
                self.min_travel_ratio
            } else {
                0.0
            },
            mean_crossing_events: f64::from(self.crossing_events_sum) / denom,
            crossing_permit_rate: if self.crossing_events_sum == 0 {
                0.0
            } else {
                f64::from(self.crossing_permit_sum) / f64::from(self.crossing_events_sum)
            },
            mean_crossing_bribes: f64::from(self.crossing_bribe_attempt_sum) / denom,
            crossing_bribe_success_rate: if self.crossing_bribe_attempt_sum == 0 {
                0.0
            } else {
                f64::from(self.crossing_bribe_success_sum)
                    / f64::from(self.crossing_bribe_attempt_sum)
            },
            mean_crossing_detours: f64::from(self.crossing_detour_sum) / denom,
            crossing_failure_rate: if self.crossing_events_sum == 0 {
                0.0
            } else {
                f64::from(self.crossing_failure_sum) / f64::from(self.crossing_events_sum)
            },
        }
    }
}

#[derive(Debug, Default, Clone)]
struct RunningStats {
    count: u32,
    mean: f64,
    m2: f64,
}

impl RunningStats {
    fn add(&mut self, value: f64) {
        self.count += 1;
        let count = f64::from(self.count);
        let delta = value - self.mean;
        self.mean += delta / count;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    const fn mean(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.mean }
    }

    fn variance(&self) -> f64 {
        if self.count > 1 {
            self.m2 / f64::from(self.count - 1)
        } else {
            0.0
        }
    }

    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
}

fn add_expectations(
    plan: crate::logic::SimulationPlan,
    strategy: GameplayStrategy,
) -> crate::logic::SimulationPlan {
    match strategy {
        GameplayStrategy::Balanced => plan.with_expectation(full_game_balanced_expectation),
        GameplayStrategy::Conservative => plan.with_expectation(full_game_conservative_expectation),
        GameplayStrategy::Aggressive => plan.with_expectation(full_game_aggressive_expectation),
        GameplayStrategy::ResourceManager => {
            plan.with_expectation(full_game_resource_manager_expectation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::game_tester::FailureFamily;
    use crate::logic::reports::generate_csv_report;
    use crate::logic::seeds::SeedInfo;
    use crate::logic::{GameTester, TesterAssets, run_playability_analysis};
    use dystrail_game::data::EncounterData;
    use dystrail_game::state::Season;
    use dystrail_game::{
        CrossingKind, CrossingOutcomeTelemetry, CrossingTelemetry, GameMode, Region,
    };
    use sha2::{Digest, Sha256};
    use std::collections::{BTreeMap, HashSet};
    use std::path::PathBuf;
    use std::sync::Arc;

    const JOURNEY_LEDGER_DIGEST: [u8; 32] = [
        84, 205, 219, 244, 107, 113, 197, 255, 64, 247, 230, 191, 183, 131, 195, 94, 141, 104, 255,
        28, 149, 0, 126, 193, 91, 134, 174, 41, 196, 158, 36, 163,
    ];
    const CSV_DIGEST_BASELINE: [u8; 32] = [
        192, 29, 241, 192, 222, 94, 1, 55, 83, 187, 172, 130, 84, 144, 105, 166, 36, 220, 12, 20,
        177, 154, 36, 41, 182, 211, 31, 92, 69, 164, 110, 70,
    ];

    const TEST_SCENARIOS: &[(GameMode, GameplayStrategy)] = &[
        (GameMode::Classic, GameplayStrategy::Balanced),
        (GameMode::Classic, GameplayStrategy::ResourceManager),
        (GameMode::Deep, GameplayStrategy::Balanced),
        (GameMode::Deep, GameplayStrategy::Conservative),
        (GameMode::Deep, GameplayStrategy::Aggressive),
    ];

    fn tester(verbose: bool) -> GameTester {
        GameTester::new(Arc::new(TesterAssets::load_default()), verbose)
    }

    #[test]
    fn generates_records_for_each_scenario() {
        let seeds = vec![SeedInfo::from_numeric(1337)];
        let records = run_playability_analysis(&tester(false), &seeds, 1).unwrap();
        assert_eq!(records.len(), PLAYABILITY_SCENARIOS.len());
        assert!(records.iter().all(|r| !r.seed_code.is_empty()));
    }

    #[test]
    fn aggregates_match_record_counts() {
        let seeds = vec![SeedInfo::from_numeric(1337)];
        let iterations = 3;
        let records = run_playability_analysis(&tester(false), &seeds, iterations).unwrap();
        let aggregates = aggregate_playability(&records);

        let scenario_name = "Classic - Balanced";
        let agg = aggregates
            .iter()
            .find(|a| a.scenario_name == scenario_name)
            .expect("aggregate for Classic Balanced");

        let matching: Vec<_> = records
            .iter()
            .filter(|r| r.scenario_name == scenario_name)
            .collect();
        assert_eq!(agg.iterations, matching.len());

        let boss_reached: u32 = matching
            .iter()
            .filter(|r| r.metrics.boss.reached)
            .count()
            .try_into()
            .unwrap();
        let boss_won: u32 = matching
            .iter()
            .filter(|r| r.metrics.boss.won)
            .count()
            .try_into()
            .unwrap();

        let iterations_u32: u32 = matching.len().try_into().unwrap();
        let denom = if iterations_u32 == 0 {
            1.0
        } else {
            f64::from(iterations_u32)
        };
        let reach_ratio = f64::from(boss_reached) / denom;
        let win_ratio = f64::from(boss_won) / denom;

        let epsilon = 1e-6;
        assert!((agg.boss_reach_pct - reach_ratio).abs() < epsilon);
        assert!((agg.boss_win_pct - win_ratio).abs() < epsilon);
    }

    #[test]
    fn emit_record_warnings_covers_crossing_anomalies() {
        let mut warn_counts = BTreeMap::new();
        let mut metrics = PlayabilityMetrics::default();
        metrics.crossing_events = vec![
            CrossingTelemetry {
                day: 1,
                region: Region::Heartland,
                season: Season::Winter,
                kind: CrossingKind::Checkpoint,
                permit_used: false,
                bribe_attempted: true,
                bribe_success: None,
                bribe_cost_cents: 0,
                bribe_chance: None,
                bribe_roll: None,
                detour_taken: true,
                detour_days: None,
                detour_base_supplies_delta: None,
                detour_extra_supplies_loss: None,
                detour_pants_delta: None,
                terminal_threshold: 0.0,
                terminal_roll: None,
                outcome: CrossingOutcomeTelemetry::Passed,
            },
            CrossingTelemetry {
                day: 2,
                region: Region::Heartland,
                season: Season::Winter,
                kind: CrossingKind::Checkpoint,
                permit_used: false,
                bribe_attempted: false,
                bribe_success: Some(true),
                bribe_cost_cents: 0,
                bribe_chance: None,
                bribe_roll: None,
                detour_taken: false,
                detour_days: None,
                detour_base_supplies_delta: None,
                detour_extra_supplies_loss: None,
                detour_pants_delta: None,
                terminal_threshold: 0.0,
                terminal_roll: None,
                outcome: CrossingOutcomeTelemetry::Detoured,
            },
            CrossingTelemetry {
                day: 3,
                region: Region::Heartland,
                season: Season::Winter,
                kind: CrossingKind::Checkpoint,
                permit_used: false,
                bribe_attempted: false,
                bribe_success: None,
                bribe_cost_cents: 0,
                bribe_chance: None,
                bribe_roll: None,
                detour_taken: true,
                detour_days: Some(1),
                detour_base_supplies_delta: None,
                detour_extra_supplies_loss: None,
                detour_pants_delta: None,
                terminal_threshold: 0.0,
                terminal_roll: None,
                outcome: CrossingOutcomeTelemetry::Failed,
            },
        ];
        let record = PlayabilityRecord {
            scenario_name: "Classic - Balanced".to_string(),
            mode: GameMode::Classic,
            strategy: GameplayStrategy::Balanced,
            seed_code: "CL-TEST01".to_string(),
            seed_value: 4,
            metrics,
        };
        emit_record_warnings(&record, &mut warn_counts);
        assert!(!warn_counts.is_empty());
    }

    #[test]
    fn encounter_catalog_contains_rotation_additions() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("dystrail-web")
            .join("static")
            .join("assets")
            .join("data")
            .join("game.json");
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let data = EncounterData::from_json(&json).expect("parse game.json");
        let ids: HashSet<_> = data.encounters.iter().map(|enc| enc.id.as_str()).collect();
        for expected in ["classic_civic_potluck", "deep_watchdog_sync"] {
            assert!(
                ids.contains(expected),
                "expected encounter id {expected} to be present in game.json"
            );
        }
    }

    #[test]
    fn otdeluxe_parity_invariants_pass() {
        validate_otdeluxe_parity_invariants().expect("parity invariants ok");
    }

    #[test]
    fn validate_targets_accepts_balanced_sample_data() {
        let (aggregates, records) = satisfied_data();
        assert!(validate_playability_targets(&aggregates, &records).is_ok());
    }

    #[test]
    fn deterministic_csv_digest_for_fixed_seed() {
        let seeds = vec![SeedInfo::from_numeric(4242)];
        let records = run_playability_analysis(&tester(false), &seeds, 1).unwrap();
        let digest = csv_digest(&records);
        assert_eq!(
            digest, CSV_DIGEST_BASELINE,
            "canonical CSV digest drifted; update baseline if intentional"
        );
    }

    #[test]
    fn deterministic_journey_digest_baseline() {
        let digest = journey_digest(0xDEAD_BEEF);
        assert_eq!(
            digest, JOURNEY_LEDGER_DIGEST,
            "journey ledger digest drifted; update baseline if intentional"
        );
    }

    #[test]
    fn validate_crossing_determinism_rejects_failed_bribe_success() {
        let mut record = base_record(GameMode::Classic, GameplayStrategy::Balanced);
        record.metrics.crossing_events = vec![CrossingTelemetry {
            day: 1,
            region: Region::Heartland,
            season: Season::Summer,
            kind: CrossingKind::Checkpoint,
            permit_used: false,
            bribe_attempted: true,
            bribe_success: Some(true),
            bribe_cost_cents: 0,
            bribe_chance: None,
            bribe_roll: None,
            detour_taken: false,
            detour_days: None,
            detour_base_supplies_delta: None,
            detour_extra_supplies_loss: None,
            detour_pants_delta: None,
            terminal_threshold: 0.0,
            terminal_roll: None,
            outcome: CrossingOutcomeTelemetry::Failed,
        }];
        let err = validate_crossing_determinism(&[record]).expect_err("should fail");
        assert!(err.to_string().contains("Crossing determinism violated"));
    }

    #[test]
    fn ensure_crossing_consistency_rejects_mixed_outcomes() {
        let mut first = base_record(GameMode::Classic, GameplayStrategy::Balanced);
        let mut second = base_record(GameMode::Classic, GameplayStrategy::Balanced);
        first.seed_value = 42;
        second.seed_value = 42;
        first.metrics.crossing_events = vec![CrossingTelemetry {
            day: 1,
            region: Region::Heartland,
            season: Season::Summer,
            kind: CrossingKind::Checkpoint,
            permit_used: false,
            bribe_attempted: false,
            bribe_success: None,
            bribe_cost_cents: 0,
            bribe_chance: None,
            bribe_roll: None,
            detour_taken: false,
            detour_days: None,
            detour_base_supplies_delta: None,
            detour_extra_supplies_loss: None,
            detour_pants_delta: None,
            terminal_threshold: 0.0,
            terminal_roll: None,
            outcome: CrossingOutcomeTelemetry::Failed,
        }];
        second.metrics.crossing_events = vec![CrossingTelemetry {
            day: 1,
            region: Region::Heartland,
            season: Season::Summer,
            kind: CrossingKind::Checkpoint,
            permit_used: false,
            bribe_attempted: false,
            bribe_success: None,
            bribe_cost_cents: 0,
            bribe_chance: None,
            bribe_roll: None,
            detour_taken: false,
            detour_days: None,
            detour_base_supplies_delta: None,
            detour_extra_supplies_loss: None,
            detour_pants_delta: None,
            terminal_threshold: 0.0,
            terminal_roll: None,
            outcome: CrossingOutcomeTelemetry::Detoured,
        }];

        let err = ensure_crossing_consistency(&[first, second]).expect_err("should fail");
        assert!(err.to_string().contains("mixed outcomes"));
    }

    #[test]
    fn aggregate_playability_tracks_pants_failure() {
        let mut record = base_record(GameMode::Classic, GameplayStrategy::Balanced);
        record.metrics.final_pants = 120;
        record.metrics.ending_type = "Pants Failure".to_string();
        let aggregates = aggregate_playability(&[record]);
        let aggregate = aggregates
            .iter()
            .find(|agg| agg.scenario_name == "Classic - Balanced")
            .expect("aggregate");
        assert!(aggregate.pants_failure_pct > 0.0);
    }

    fn satisfied_data() -> (Vec<PlayabilityAggregate>, Vec<PlayabilityRecord>) {
        let aggregates = TEST_SCENARIOS
            .iter()
            .map(|&(mode, strategy)| base_aggregate(mode, strategy))
            .collect();
        let records = TEST_SCENARIOS
            .iter()
            .map(|&(mode, strategy)| base_record(mode, strategy))
            .collect();
        (aggregates, records)
    }

    fn base_record(mode: GameMode, strategy: GameplayStrategy) -> PlayabilityRecord {
        let scenario_name = format!("{} - {}", mode_label(mode), strategy);
        PlayabilityRecord {
            scenario_name,
            mode,
            strategy,
            seed_code: "CL-TEST00".to_string(),
            seed_value: 0,
            metrics: base_metrics(mode),
        }
    }

    fn base_metrics(mode: GameMode) -> PlayabilityMetrics {
        let mut metrics = PlayabilityMetrics::default();
        metrics.days_survived = 120;
        metrics.miles_traveled = 2_000.0;
        metrics.travel_days = 100;
        metrics.partial_travel_days = 10;
        metrics.non_travel_days = 5;
        metrics.avg_miles_per_day = 15.0;
        metrics.unique_encounters = 40;
        metrics.rotation_events = 6;
        metrics.travel_ratio = 0.75;
        metrics.unique_per_20_days = if mode.is_deep() { 1.2 } else { 1.8 };
        metrics.milestones.reached_2000_by_day150 = true;
        metrics.crossing_events.clear();
        metrics.crossing_permit_uses = 0;
        metrics.crossing_bribe_attempts = 0;
        metrics.crossing_bribe_successes = 0;
        metrics.crossing_detours_taken = 0;
        metrics.crossing_failures = 0;
        metrics.milestones.survived = true;
        metrics.failure_family = Some(FailureFamily::Vehicle);
        metrics
    }

    fn base_aggregate(mode: GameMode, strategy: GameplayStrategy) -> PlayabilityAggregate {
        let scenario_name = format!("{} - {}", mode_label(mode), strategy);
        PlayabilityAggregate {
            scenario_name,
            mode,
            strategy,
            iterations: 10,
            mean_days: 120.0,
            std_days: 3.0,
            mean_miles: 2_000.0,
            std_miles: 25.0,
            boss_reach_pct: 0.1,
            boss_win_pct: 0.05,
            pants_failure_pct: 0.1,
            mean_travel_ratio: 0.7,
            mean_unique_per_20: 1.2,
            mean_rotation_events: 5.0,
            pct_reached_2k_by_150: 0.2,
            min_unique_per_20: 0.9,
            min_travel_ratio: 0.6,
            mean_crossing_events: 1.0,
            crossing_permit_rate: 0.25,
            mean_crossing_bribes: 0.0,
            crossing_bribe_success_rate: 0.0,
            mean_crossing_detours: 0.2,
            crossing_failure_rate: 0.05,
        }
    }

    fn csv_digest(records: &[PlayabilityRecord]) -> [u8; 32] {
        let mut buffer = Vec::new();
        generate_csv_report(&mut buffer, records).expect("csv serialization");
        let mut hasher = Sha256::new();
        hasher.update(buffer);
        let digest = hasher.finalize();
        let mut bytes = [0_u8; 32];
        bytes.copy_from_slice(&digest);
        bytes
    }

    fn journey_digest(seed: u64) -> [u8; 32] {
        use dystrail_game::journey::JourneySession;
        use dystrail_game::{EndgameTravelCfg, GameMode, StrategyId, TravelDayKind};
        let encounters = EncounterData::empty();
        let mut session = JourneySession::new(
            GameMode::Classic,
            StrategyId::Balanced,
            seed,
            encounters,
            &EndgameTravelCfg::default_config(),
        );
        for _ in 0..120 {
            let outcome = session.tick_day();
            if outcome.ended {
                break;
            }
        }
        let state = session.into_state();
        let mut buffer = String::new();
        for record in &state.day_records {
            use std::fmt::Write as _;
            let kind = match record.kind {
                TravelDayKind::Travel => "travel",
                TravelDayKind::Partial => "partial",
                TravelDayKind::NonTravel => "non_travel",
            };
            writeln!(buffer, "{},{:.3},{}", record.day_index, record.miles, kind)
                .expect("write day record");
        }
        let mut hasher = Sha256::new();
        hasher.update(buffer.as_bytes());
        let digest = hasher.finalize();
        let mut bytes = [0_u8; 32];
        bytes.copy_from_slice(&digest);
        bytes
    }
}
