use anyhow::{Context, Result, ensure};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::TryFrom;
use std::sync::OnceLock;

use crate::common::scenario::full_game::{
    full_game_aggressive_expectation, full_game_balanced_expectation,
    full_game_conservative_expectation, full_game_monte_carlo_expectation, full_game_plan,
    full_game_resource_manager_expectation,
};
use crate::logic::game_tester::FailureFamily;
use crate::logic::seeds::SeedInfo;
use crate::logic::{GameTester, GameplayStrategy, PlayabilityMetrics};
use dystrail_game::GameMode;
use dystrail_game::journey::{AcceptanceGuards, JourneyController, PolicyId, StrategyId};
use dystrail_game::state::CrossingOutcomeTelemetry;

const AVG_MPD_MIN: f64 = 10.0;
const AVG_MPD_MAX: f64 = 20.0;
const CLASSIC_CROSSING_FAILURE_MAX: f64 = 0.12;
const DEEP_CROSSING_FAILURE_MAX: f64 = 0.16;
const DISTANCE_DRIFT_PCT: f64 = 0.05;
const CLASSIC_BALANCED_BOSS_REACH_MIN: f64 = 0.30;
const CLASSIC_BALANCED_BOSS_REACH_MAX: f64 = 0.50;
const CLASSIC_BALANCED_BOSS_WIN_MIN: f64 = 0.20;
const CLASSIC_BALANCED_BOSS_WIN_MAX: f64 = 0.35;
const CLASSIC_BALANCED_SURVIVAL_MIN: f64 = 0.60;
const CLASSIC_BALANCED_SURVIVAL_MAX: f64 = 0.80;
const FAILURE_FAMILY_MAX_SHARE: f64 = 0.50;
const CONSERVATIVE_BOSS_WIN_WARN: f64 = 0.40;
const DEEP_CROSSING_WARN_MIN: f64 = 0.08;
const DEEP_CROSSING_WARN_MAX: f64 = 0.18;
const DEEP_BOSS_REACH_WARN_MIN: f64 = CLASSIC_BALANCED_BOSS_REACH_MIN * 0.5;
const DEEP_BOSS_REACH_WARN_MAX: f64 = CLASSIC_BALANCED_BOSS_REACH_MAX * 1.5;
const DEEP_BOSS_WIN_WARN_MIN: f64 = CLASSIC_BALANCED_BOSS_WIN_MIN * 0.5;
const DEEP_BOSS_WIN_WARN_MAX: f64 = CLASSIC_BALANCED_BOSS_WIN_MAX * 1.5;
const MONTE_CARLO_VARIANCE_EPS: f64 = 1e-6;

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
    pub mean_avg_mpd: f64,
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
    pub mean_stop_cap_conversions: f64,
    pub endgame_activation_rate: f64,
    pub endgame_field_repair_rate: f64,
    pub mean_endgame_cooldown: f64,
    pub survival_rate: f64,
    pub failure_vehicle_pct: f64,
    pub failure_sanity_pct: f64,
    pub failure_exposure_pct: f64,
    pub failure_crossing_pct: f64,
}

type GuardKey = (PolicyId, StrategyId);

fn guard_registry() -> &'static HashMap<GuardKey, AcceptanceGuards> {
    static REGISTRY: OnceLock<HashMap<GuardKey, AcceptanceGuards>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut map = HashMap::new();
        for &policy in &[PolicyId::Classic, PolicyId::Deep] {
            for &strategy in &[
                StrategyId::Balanced,
                StrategyId::Aggressive,
                StrategyId::Conservative,
                StrategyId::ResourceManager,
                StrategyId::MonteCarlo,
            ] {
                let controller = JourneyController::new(policy, strategy, 0);
                map.insert((policy, strategy), controller.config().guards.clone());
            }
        }
        map
    })
}

fn scenario_guards(mode: GameMode, strategy: GameplayStrategy) -> AcceptanceGuards {
    let policy = PolicyId::from(mode);
    let strategy_id = to_strategy_id(strategy);
    guard_registry()
        .get(&(policy, strategy_id))
        .cloned()
        .unwrap_or_else(AcceptanceGuards::default)
}

const fn to_strategy_id(strategy: GameplayStrategy) -> StrategyId {
    match strategy {
        GameplayStrategy::Balanced => StrategyId::Balanced,
        GameplayStrategy::Aggressive => StrategyId::Aggressive,
        GameplayStrategy::Conservative => StrategyId::Conservative,
        GameplayStrategy::ResourceManager => StrategyId::ResourceManager,
        GameplayStrategy::MonteCarlo => StrategyId::MonteCarlo,
    }
}

fn guard_distance_window(guard: &AcceptanceGuards) -> (f64, f64) {
    let target = f64::from(guard.target_distance);
    let spread = target * DISTANCE_DRIFT_PCT;
    (target - spread, target + spread)
}

fn enforce_acceptance_guards(agg: &PlayabilityAggregate, guard: &AcceptanceGuards) -> Result<()> {
    let ratio_floor = f64::from(guard.min_travel_ratio);
    ensure!(
        agg.mean_travel_ratio >= ratio_floor,
        "{} travel ratio {:.1}% below {:.1}% target",
        agg.scenario_name,
        agg.mean_travel_ratio * 100.0,
        ratio_floor * 100.0
    );
    let (dist_min, dist_max) = guard_distance_window(guard);
    ensure!(
        agg.mean_miles >= dist_min,
        "{} average mileage {:.0} below {:.0} mi guard",
        agg.scenario_name,
        agg.mean_miles,
        dist_min
    );
    ensure!(
        agg.mean_miles <= dist_max,
        "{} average mileage {:.0} exceeds {:.0} mi guard",
        agg.scenario_name,
        agg.mean_miles,
        dist_max
    );
    let days_min = f64::from(guard.target_days_min);
    let days_max = f64::from(guard.target_days_max);
    ensure!(
        agg.mean_days >= days_min,
        "{} average duration {:.1}d below {:.1}d guard",
        agg.scenario_name,
        agg.mean_days,
        days_min
    );
    ensure!(
        agg.mean_days <= days_max,
        "{} average duration {:.1}d exceeds {:.1}d guard",
        agg.scenario_name,
        agg.mean_days,
        days_max
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
    let guard = scenario_guards(record.mode, record.strategy);
    let ratio_floor = f64::from(guard.min_travel_ratio);
    if record.metrics.travel_ratio < ratio_floor {
        push_limited_warn(
            warn_counts,
            &format!("{}::travel_ratio", record.scenario_name),
            3,
            || {
                format!(
                    "WARN: {} seed {} travel ratio {:.1}% < {:.1}%",
                    record.scenario_name,
                    record.seed_code,
                    record.metrics.travel_ratio * 100.0,
                    ratio_floor * 100.0
                )
            },
        );
    }

    warn_deep_conservative(record, warn_counts);
    warn_deep_aggressive(record, warn_counts);
    warn_crossing_anomalies(record, warn_counts);
}

fn warn_deep_conservative(record: &PlayabilityRecord, warn_counts: &mut BTreeMap<String, usize>) {
    if record.scenario_name != "Deep - Conservative" {
        return;
    }

    if !record.metrics.reached_2000_by_day150 {
        push_limited_warn(
            warn_counts,
            &format!("{}::deep_conservative_2k", record.scenario_name),
            5,
            || {
                format!(
                    "WARN: Deep Conservative seed {} failed ≥2k@150",
                    record.seed_code
                )
            },
        );
    }
    if record.metrics.final_pants >= 100 {
        push_limited_warn(
            warn_counts,
            &format!("{}::deep_conservative_pants", record.scenario_name),
            3,
            || {
                format!(
                    "WARN: Deep Conservative seed {} ended via pants emergency",
                    record.seed_code
                )
            },
        );
    }
    if record.metrics.unique_per_20_days < 1.60 {
        push_limited_warn(
            warn_counts,
            &format!("{}::deep_conservative_unique", record.scenario_name),
            3,
            || {
                format!(
                    "WARN: Deep Conservative seed {} unique encounters per 20d {:.2} < 1.60",
                    record.seed_code, record.metrics.unique_per_20_days
                )
            },
        );
    }
}

fn warn_deep_aggressive(record: &PlayabilityRecord, warn_counts: &mut BTreeMap<String, usize>) {
    if record.scenario_name != "Deep - Aggressive" {
        return;
    }

    if !record.metrics.boss_reached {
        push_limited_warn(
            warn_counts,
            &format!("{}::deep_aggressive_reach", record.scenario_name),
            5,
            || {
                format!(
                    "WARN: Deep Aggressive seed {} failed to reach the boss",
                    record.seed_code
                )
            },
        );
    } else if !record.metrics.boss_won {
        push_limited_warn(
            warn_counts,
            &format!("{}::deep_aggressive_win", record.scenario_name),
            5,
            || {
                format!(
                    "WARN: Deep Aggressive seed {} reached boss but did not win",
                    record.seed_code
                )
            },
        );
    }
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

fn emit_aggregate_warnings(aggregates: &[PlayabilityAggregate]) {
    for agg in aggregates {
        if agg.mean_crossing_bribes > 0.0 && agg.crossing_bribe_success_rate <= 0.70 {
            println!(
                "WARN: {} crossing bribe success {:.1}% ≤ 70%",
                agg.scenario_name,
                agg.crossing_bribe_success_rate * 100.0
            );
        }
        if agg.crossing_failure_rate > 0.12 {
            println!(
                "WARN: {} terminal crossing rate {:.1}% > 12%",
                agg.scenario_name,
                agg.crossing_failure_rate * 100.0
            );
        }
        if agg.mean_stop_cap_conversions > 1.2 {
            println!(
                "WARN: {} average stop-cap conversions {:.2} > 1.2",
                agg.scenario_name, agg.mean_stop_cap_conversions
            );
        }
        if agg.mode == GameMode::Deep
            && matches!(agg.strategy, GameplayStrategy::Aggressive)
            && agg.endgame_field_repair_rate < 1.0
        {
            println!(
                "WARN: {} field repair only triggered {:.1}% of runs",
                agg.scenario_name,
                agg.endgame_field_repair_rate * 100.0
            );
        }
        if agg.endgame_activation_rate == 0.0
            && agg.mode == GameMode::Deep
            && matches!(
                agg.strategy,
                GameplayStrategy::Balanced | GameplayStrategy::Aggressive
            )
        {
            println!(
                "WARN: {} endgame controller never activated",
                agg.scenario_name
            );
        }
        if agg.mean_endgame_cooldown > 0.0 {
            println!(
                "WARN: {} average endgame cooldown {:.2} days remaining",
                agg.scenario_name, agg.mean_endgame_cooldown
            );
        }
    }
}

fn find_aggregate<'a>(
    aggregates: &'a [PlayabilityAggregate],
    name: &str,
) -> Result<&'a PlayabilityAggregate> {
    aggregates
        .iter()
        .find(|agg| agg.scenario_name == name)
        .with_context(|| format!("missing playability summary for {name}"))
}

fn get_aggregate<'a>(
    aggregates: &'a [PlayabilityAggregate],
    name: &str,
) -> Option<&'a PlayabilityAggregate> {
    aggregates.iter().find(|agg| agg.scenario_name == name)
}

fn validate_classic_balanced(agg: &PlayabilityAggregate) -> Result<()> {
    ensure!(
        agg.min_unique_per_20 >= 2.0,
        "Classic Balanced min unique encounters per 20 days {:.2} below 2.0 requirement",
        agg.min_unique_per_20
    );
    ensure!(
        agg.mean_unique_per_20 >= 2.0,
        "Classic Balanced mean unique encounters per 20 days {:.2} below 2.0 target",
        agg.mean_unique_per_20
    );
    ensure!(
        agg.pct_reached_2k_by_150 >= 0.25,
        "Classic Balanced reached 2,000 miles by day 150 {:.1}% < 25% threshold",
        agg.pct_reached_2k_by_150 * 100.0
    );
    ensure!(
        agg.boss_reach_pct >= CLASSIC_BALANCED_BOSS_REACH_MIN,
        "Classic Balanced boss reach {:.1}% below {:.0}% floor",
        agg.boss_reach_pct * 100.0,
        CLASSIC_BALANCED_BOSS_REACH_MIN * 100.0
    );
    ensure!(
        agg.boss_reach_pct <= CLASSIC_BALANCED_BOSS_REACH_MAX,
        "Classic Balanced boss reach {:.1}% exceeds {:.0}% ceiling",
        agg.boss_reach_pct * 100.0,
        CLASSIC_BALANCED_BOSS_REACH_MAX * 100.0
    );
    ensure!(
        agg.boss_win_pct >= CLASSIC_BALANCED_BOSS_WIN_MIN,
        "Classic Balanced boss win {:.1}% below {:.0}% floor",
        agg.boss_win_pct * 100.0,
        CLASSIC_BALANCED_BOSS_WIN_MIN * 100.0
    );
    ensure!(
        agg.boss_win_pct <= CLASSIC_BALANCED_BOSS_WIN_MAX,
        "Classic Balanced boss win {:.1}% exceeds {:.0}% ceiling",
        agg.boss_win_pct * 100.0,
        CLASSIC_BALANCED_BOSS_WIN_MAX * 100.0
    );
    ensure!(
        agg.survival_rate >= CLASSIC_BALANCED_SURVIVAL_MIN,
        "Classic Balanced survival {:.1}% below {:.0}% floor",
        agg.survival_rate * 100.0,
        CLASSIC_BALANCED_SURVIVAL_MIN * 100.0
    );
    ensure!(
        agg.survival_rate <= CLASSIC_BALANCED_SURVIVAL_MAX,
        "Classic Balanced survival {:.1}% exceeds {:.0}% ceiling",
        agg.survival_rate * 100.0,
        CLASSIC_BALANCED_SURVIVAL_MAX * 100.0
    );
    let dominant_failure = agg
        .failure_vehicle_pct
        .max(agg.failure_sanity_pct)
        .max(agg.failure_exposure_pct)
        .max(agg.failure_crossing_pct);
    ensure!(
        dominant_failure <= FAILURE_FAMILY_MAX_SHARE,
        "Classic Balanced failure mix skewed: {:.1}% > {:.0}% cap",
        dominant_failure * 100.0,
        FAILURE_FAMILY_MAX_SHARE * 100.0
    );
    Ok(())
}

fn validate_deep_balanced(agg: &PlayabilityAggregate) -> Result<()> {
    ensure!(
        agg.mean_unique_per_20 >= 1.5,
        "Deep Balanced mean unique encounters per 20 days {:.2} below 1.5 target",
        agg.mean_unique_per_20
    );
    ensure!(
        agg.min_unique_per_20 >= 1.5,
        "Deep Balanced min unique encounters per 20 days {:.2} below 1.5 requirement",
        agg.min_unique_per_20
    );
    ensure!(
        agg.pct_reached_2k_by_150 >= 0.25,
        "Deep Balanced reached 2,000 miles by day 150 {:.1}% < 25% threshold",
        agg.pct_reached_2k_by_150 * 100.0
    );
    warn_deep_boss_rates(agg);
    Ok(())
}

fn validate_deep_conservative(agg: &PlayabilityAggregate) -> Result<()> {
    ensure!(
        agg.pct_reached_2k_by_150 >= 0.25,
        "Deep Conservative ≥2k@150 {:.1}% < 25% threshold",
        agg.pct_reached_2k_by_150 * 100.0
    );
    ensure!(
        agg.pants_failure_pct <= 0.30,
        "Deep Conservative pants failure rate {:.1}% exceeds 30% cap",
        agg.pants_failure_pct * 100.0
    );
    ensure!(
        agg.mean_travel_ratio >= 0.90,
        "Deep Conservative travel ratio {:.1}% below 90% target",
        agg.mean_travel_ratio * 100.0
    );
    warn_deep_boss_rates(agg);
    Ok(())
}

fn validate_deep_aggressive(agg: &PlayabilityAggregate) -> Result<()> {
    ensure!(
        agg.boss_reach_pct >= 0.65,
        "Deep Aggressive boss reach {:.1}% below 65% target",
        agg.boss_reach_pct * 100.0
    );
    ensure!(
        agg.boss_win_pct >= 0.02,
        "Deep Aggressive boss win {:.1}% below 2% target",
        agg.boss_win_pct * 100.0
    );
    ensure!(
        agg.mean_miles >= 1980.0,
        "Deep Aggressive average mileage {:.0} below 1980 mi goal",
        agg.mean_miles
    );
    ensure!(
        agg.pct_reached_2k_by_150 >= 0.70,
        "Deep Aggressive ≥2k@150 {:.1}% < 70% threshold",
        agg.pct_reached_2k_by_150 * 100.0
    );
    warn_deep_boss_rates(agg);
    Ok(())
}

fn emit_classic_strategy_warnings(
    aggregates: &[PlayabilityAggregate],
    classic_balanced: &PlayabilityAggregate,
) {
    if let Some(aggressive) = get_aggregate(aggregates, "Classic - Aggressive") {
        if aggressive.survival_rate > CLASSIC_BALANCED_SURVIVAL_MAX {
            println!(
                "WARN: Classic Aggressive survival {:.1}% exceeds Balanced upper band",
                aggressive.survival_rate * 100.0
            );
        }
        if aggressive.boss_win_pct > CLASSIC_BALANCED_BOSS_WIN_MAX {
            println!(
                "WARN: Classic Aggressive boss win {:.1}% exceeds Balanced ceiling",
                aggressive.boss_win_pct * 100.0
            );
        }
    }
    for scenario in ["Classic - Conservative", "Classic - Resource Manager"] {
        if let Some(agg) = get_aggregate(aggregates, scenario)
            && agg.boss_win_pct > CONSERVATIVE_BOSS_WIN_WARN
        {
            println!(
                "WARN: {} boss win {:.1}% exceeds {:.0}% cozy cap",
                scenario,
                agg.boss_win_pct * 100.0,
                CONSERVATIVE_BOSS_WIN_WARN * 100.0
            );
        }
    }
    if let Some(monte_carlo) = get_aggregate(aggregates, "Classic - Monte Carlo")
        && (monte_carlo.std_miles <= classic_balanced.std_miles + MONTE_CARLO_VARIANCE_EPS
            || monte_carlo.std_days <= classic_balanced.std_days + MONTE_CARLO_VARIANCE_EPS)
    {
        println!(
            "WARN: Classic Monte Carlo variance too low (days std {:.2}, miles std {:.2})",
            monte_carlo.std_days, monte_carlo.std_miles
        );
    }
}

fn warn_deep_boss_rates(agg: &PlayabilityAggregate) {
    if agg.boss_reach_pct < DEEP_BOSS_REACH_WARN_MIN {
        println!(
            "WARN: {} boss reach {:.1}% below {:.0}% deep lower bound",
            agg.scenario_name,
            agg.boss_reach_pct * 100.0,
            DEEP_BOSS_REACH_WARN_MIN * 100.0
        );
    } else if agg.boss_reach_pct > DEEP_BOSS_REACH_WARN_MAX {
        println!(
            "WARN: {} boss reach {:.1}% exceeds {:.0}% deep upper bound",
            agg.scenario_name,
            agg.boss_reach_pct * 100.0,
            DEEP_BOSS_REACH_WARN_MAX * 100.0
        );
    }
    if agg.boss_win_pct < DEEP_BOSS_WIN_WARN_MIN {
        println!(
            "WARN: {} boss win {:.1}% below {:.0}% deep lower bound",
            agg.scenario_name,
            agg.boss_win_pct * 100.0,
            DEEP_BOSS_WIN_WARN_MIN * 100.0
        );
    } else if agg.boss_win_pct > DEEP_BOSS_WIN_WARN_MAX {
        println!(
            "WARN: {} boss win {:.1}% exceeds {:.0}% deep upper bound",
            agg.scenario_name,
            agg.boss_win_pct * 100.0,
            DEEP_BOSS_WIN_WARN_MAX * 100.0
        );
    }
}

const PLAYABILITY_SCENARIOS: &[(GameMode, GameplayStrategy)] = &[
    (GameMode::Classic, GameplayStrategy::Balanced),
    (GameMode::Classic, GameplayStrategy::Conservative),
    (GameMode::Classic, GameplayStrategy::Aggressive),
    (GameMode::Classic, GameplayStrategy::ResourceManager),
    (GameMode::Classic, GameplayStrategy::MonteCarlo),
    (GameMode::Deep, GameplayStrategy::Balanced),
    (GameMode::Deep, GameplayStrategy::Conservative),
    (GameMode::Deep, GameplayStrategy::Aggressive),
    (GameMode::Deep, GameplayStrategy::ResourceManager),
    (GameMode::Deep, GameplayStrategy::MonteCarlo),
];

pub fn run_playability_analysis(
    seeds: &[SeedInfo],
    iterations: usize,
    verbose: bool,
) -> Result<Vec<PlayabilityRecord>> {
    let iterations = iterations.max(1);
    let tester = GameTester::try_new(verbose);
    let mut records = Vec::with_capacity(seeds.len() * PLAYABILITY_SCENARIOS.len() * iterations);

    for &(mode, strategy) in PLAYABILITY_SCENARIOS {
        for seed in seeds.iter().filter(|seed| seed.matches_mode(mode)) {
            for iteration in 0..iterations {
                let iteration_offset = u64::try_from(iteration).unwrap_or(0);
                let iteration_seed = seed.seed.wrapping_add(iteration_offset);
                let plan = add_expectations(full_game_plan(mode, strategy), strategy);
                let summary = tester.run_plan(&plan, iteration_seed);
                for expectation in &plan.expectations {
                    expectation(&summary).with_context(|| {
                        format!(
                            "Playability expectation failed for mode {:?}, strategy {}, seed {} (iteration {})",
                            mode, strategy, seed.seed, iteration + 1
                        )
                    })?;
                }

                let metrics = summary.metrics.clone();
                let scenario_name = format!("{} - {}", mode_label(mode), strategy);
                let seed_code = dystrail_game::encode_friendly(mode.is_deep(), iteration_seed);

                records.push(PlayabilityRecord {
                    scenario_name,
                    mode,
                    strategy,
                    seed_code,
                    seed_value: iteration_seed,
                    metrics,
                });
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

    let results: Vec<_> = aggregates
        .into_values()
        .map(AggregateBuilder::finish)
        .collect();

    emit_aggregate_warnings(&results);

    results
}

pub fn validate_playability_targets(
    aggregates: &[PlayabilityAggregate],
    records: &[PlayabilityRecord],
) -> Result<()> {
    validate_record_metrics(records)?;
    validate_scenario_aggregates(aggregates)?;
    validate_global_aggregates(aggregates)?;
    validate_crossing_determinism(records)?;
    ensure_crossing_consistency(records)?;
    Ok(())
}

fn validate_record_metrics(records: &[PlayabilityRecord]) -> Result<()> {
    for record in records {
        ensure!(
            record.metrics.travel_ratio >= 0.90,
            "Travel ratio {:.1}% < 90% for scenario {} seed {}",
            record.metrics.travel_ratio * 100.0,
            record.scenario_name,
            record.seed_code
        );
        if record
            .metrics
            .ending_type
            .to_lowercase()
            .contains("vehicle")
            && record.scenario_name == "Classic - Balanced"
        {
            ensure!(
                record.metrics.miles_traveled >= 1_950.0,
                "Vehicle failure observed for scenario {} seed {} before 1950 mi (at {:.0} mi)",
                record.scenario_name,
                record.seed_code,
                record.metrics.miles_traveled
            );
        }
    }
    Ok(())
}

fn validate_scenario_aggregates(aggregates: &[PlayabilityAggregate]) -> Result<()> {
    let classic_balanced = find_aggregate(aggregates, "Classic - Balanced")?;
    validate_classic_balanced(classic_balanced)?;
    emit_classic_strategy_warnings(aggregates, classic_balanced);

    let classic_resource = find_aggregate(aggregates, "Classic - Resource Manager")?;
    ensure!(
        classic_resource.pct_reached_2k_by_150 >= 0.70,
        "Classic Resource Manager reached 2,000 miles by day 150 {:.1}% < 70% threshold",
        classic_resource.pct_reached_2k_by_150 * 100.0
    );
    ensure!(
        classic_resource.pants_failure_pct <= 0.35,
        "Classic Resource Manager pants failure rate {:.1}% exceeds 35% cap",
        classic_resource.pants_failure_pct * 100.0
    );

    let deep_balanced = find_aggregate(aggregates, "Deep - Balanced")?;
    validate_deep_balanced(deep_balanced)?;

    let deep_conservative = find_aggregate(aggregates, "Deep - Conservative")?;
    validate_deep_conservative(deep_conservative)?;

    let deep_aggressive = find_aggregate(aggregates, "Deep - Aggressive")?;
    validate_deep_aggressive(deep_aggressive)?;

    Ok(())
}

fn validate_global_aggregates(aggregates: &[PlayabilityAggregate]) -> Result<()> {
    for agg in aggregates {
        let guard = scenario_guards(agg.mode, agg.strategy);
        enforce_acceptance_guards(agg, &guard)?;
        ensure!(
            agg.mean_avg_mpd >= AVG_MPD_MIN,
            "{} average miles/day {:.2} below {:.2} floor",
            agg.scenario_name,
            agg.mean_avg_mpd,
            AVG_MPD_MIN
        );
        let avg_mpd_cap = if agg.scenario_name.ends_with("Monte Carlo") {
            AVG_MPD_MAX + 0.6
        } else {
            AVG_MPD_MAX
        };
        ensure!(
            agg.mean_avg_mpd <= avg_mpd_cap,
            "{} average miles/day {:.2} exceeds {:.2} ceiling",
            agg.scenario_name,
            agg.mean_avg_mpd,
            avg_mpd_cap
        );
        if agg.mean_crossing_bribes > 0.0 {
            ensure!(
                agg.crossing_bribe_success_rate >= 0.70,
                "{} average bribe success {:.1}% below 70% target",
                agg.scenario_name,
                agg.crossing_bribe_success_rate * 100.0
            );
        }
        if agg.mean_crossing_events > 0.0 {
            let failure_cap = failure_cap_for_mode(agg.mode);
            ensure!(
                agg.crossing_failure_rate <= failure_cap,
                "{} terminal crossing rate {:.1}% exceeds {:.0}% cap",
                agg.scenario_name,
                agg.crossing_failure_rate * 100.0,
                failure_cap * 100.0
            );
            if agg.mode == GameMode::Deep {
                if agg.crossing_failure_rate < DEEP_CROSSING_WARN_MIN {
                    println!(
                        "WARN: {} terminal crossing rate {:.1}% below {:.0}% deep lower band",
                        agg.scenario_name,
                        agg.crossing_failure_rate * 100.0,
                        DEEP_CROSSING_WARN_MIN * 100.0
                    );
                } else if agg.crossing_failure_rate > DEEP_CROSSING_WARN_MAX {
                    println!(
                        "WARN: {} terminal crossing rate {:.1}% exceeds {:.0}% deep warning band",
                        agg.scenario_name,
                        agg.crossing_failure_rate * 100.0,
                        DEEP_CROSSING_WARN_MAX * 100.0
                    );
                }
            }
        }
        ensure!(
            agg.min_travel_ratio >= 0.90,
            "{} minimum travel ratio {:.1}% below 90% floor",
            agg.scenario_name,
            agg.min_travel_ratio * 100.0
        );
    }
    Ok(())
}

fn failure_cap_for_mode(mode: GameMode) -> f64 {
    if mode == GameMode::Deep {
        DEEP_CROSSING_FAILURE_MAX
    } else {
        CLASSIC_CROSSING_FAILURE_MAX
    }
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
        GameplayStrategy::MonteCarlo => plan.with_expectation(full_game_monte_carlo_expectation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::reports::generate_csv_report;
    use crate::logic::run_playability_analysis;
    use crate::logic::seeds::SeedInfo;
    use dystrail_game::GameMode;
    use dystrail_game::data::EncounterData;
    use sha2::{Digest, Sha256};
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[test]
    fn generates_records_for_each_scenario() {
        let seeds = vec![SeedInfo::from_numeric(1337)];
        let records = run_playability_analysis(&seeds, 1, false).unwrap();
        assert_eq!(records.len(), PLAYABILITY_SCENARIOS.len());
        assert!(records.iter().all(|r| !r.seed_code.is_empty()));
    }

    #[test]
    fn balanced_samples_hit_survival_and_boss_targets() {
        let seeds: Vec<SeedInfo> = (0..120)
            .map(|i| {
                let offset = u64::try_from(i).unwrap_or(0);
                SeedInfo::for_mode(10_000_u64 + offset, GameMode::Classic)
            })
            .collect();
        let records = run_playability_analysis(&seeds, 1, false).unwrap();
        let aggregates = aggregate_playability(&records);

        let balanced_summary = aggregates
            .iter()
            .find(|agg| agg.mode == GameMode::Classic && agg.strategy == GameplayStrategy::Balanced)
            .expect("balanced summary available");
        assert!(
            balanced_summary.iterations >= 100,
            "expected at least 100 Balanced samples, observed {}",
            balanced_summary.iterations
        );
        assert!(
            (0.0..=0.60).contains(&balanced_summary.boss_win_pct),
            "boss win rate should stay near target band, observed {:.1}%",
            balanced_summary.boss_win_pct * 100.0
        );
        assert!(
            balanced_summary.pants_failure_pct <= 0.20,
            "pants failures should stay below 20%, observed {:.1}%",
            balanced_summary.pants_failure_pct * 100.0
        );

        let balanced_records: Vec<_> = records
            .iter()
            .filter(|r| r.mode == GameMode::Classic && r.strategy == GameplayStrategy::Balanced)
            .collect();
        assert!(
            !balanced_records.is_empty(),
            "balanced sample should not be empty"
        );
        let survive_120 = balanced_records
            .iter()
            .filter(|r| r.metrics.days_survived >= 120)
            .count();
        let numerator = u32::try_from(survive_120).unwrap_or(0);
        let denominator = u32::try_from(balanced_records.len()).unwrap_or(1);
        let ratio = if denominator == 0 {
            0.0
        } else {
            f64::from(numerator) / f64::from(denominator)
        };
        assert!(
            ratio <= 0.97,
            "expected ≤97% of balanced runs to reach 120 days under current tuning, observed {:.1}%",
            ratio * 100.0
        );
    }

    #[test]
    fn aggregates_match_record_counts() {
        let seeds = vec![SeedInfo::from_numeric(1337)];
        let iterations = 3;
        let records = run_playability_analysis(&seeds, iterations, false).unwrap();
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
            .filter(|r| r.metrics.boss_reached)
            .count()
            .try_into()
            .unwrap();
        let boss_won: u32 = matching
            .iter()
            .filter(|r| r.metrics.boss_won)
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
    fn validate_targets_accepts_balanced_sample_data() {
        let (aggregates, records) = satisfied_data();
        assert!(validate_playability_targets(&aggregates, &records).is_ok());
    }

    #[test]
    fn validate_targets_enforce_average_miles_per_day_band() {
        let (mut aggregates, records) = satisfied_data();
        let scenario = format!(
            "{} - {}",
            mode_label(GameMode::Deep),
            GameplayStrategy::Aggressive
        );
        let target = find_mut_aggregate(&mut aggregates, &scenario);
        target.mean_avg_mpd = 21.0;
        let err = validate_playability_targets(&aggregates, &records).unwrap_err();
        assert!(
            err.to_string().contains("average miles/day"),
            "unexpected error message: {err}"
        );
        assert!(
            err.to_string().contains(&scenario),
            "scenario name missing from error: {err}"
        );
    }

    #[test]
    fn validate_targets_enforce_distance_window() {
        let (mut aggregates, records) = satisfied_data();
        let scenario = format!(
            "{} - {}",
            mode_label(GameMode::Classic),
            GameplayStrategy::Balanced
        );
        let target = find_mut_aggregate(&mut aggregates, &scenario);
        target.mean_miles = 1_850.0;
        let err = validate_playability_targets(&aggregates, &records).unwrap_err();
        assert!(
            err.to_string().contains("average mileage"),
            "unexpected error message: {err}"
        );
        assert!(
            err.to_string().contains(&scenario),
            "expected Classic Balanced label in error: {err}"
        );
    }

    #[test]
    fn validate_targets_enforce_classic_crossing_failure_cap() {
        let (mut aggregates, records) = satisfied_data();
        let scenario = format!(
            "{} - {}",
            mode_label(GameMode::Classic),
            GameplayStrategy::ResourceManager
        );
        let target = find_mut_aggregate(&mut aggregates, &scenario);
        target.crossing_failure_rate = 0.20;
        let err = validate_playability_targets(&aggregates, &records).unwrap_err();
        assert!(
            err.to_string().contains("terminal crossing rate"),
            "unexpected error message: {err}"
        );
        assert!(
            err.to_string().contains(&scenario),
            "scenario name missing from error: {err}"
        );
        assert!(
            err.to_string().contains("12%"),
            "classic cap not referenced in error: {err}"
        );
    }

    const JOURNEY_LEDGER_DIGEST: [u8; 32] = [
        60, 185, 137, 27, 23, 11, 165, 208, 78, 158, 135, 49, 114, 180, 251, 154, 88, 231, 254, 68,
        158, 158, 117, 227, 18, 245, 56, 127, 124, 23, 129, 147,
    ];
    const CSV_DIGEST_BASELINE: [u8; 32] = [
        17, 10, 125, 238, 9, 25, 102, 120, 217, 51, 180, 167, 152, 154, 188, 244, 101, 140, 195,
        111, 102, 10, 200, 23, 216, 17, 50, 124, 95, 135, 13, 252,
    ];

    #[test]
    fn deterministic_csv_digest_for_fixed_seed() {
        let seeds = vec![SeedInfo::from_numeric(4242)];
        let records = run_playability_analysis(&seeds, 1, false).unwrap();
        let digest = csv_digest(&records);
        assert_eq!(
            digest, CSV_DIGEST_BASELINE,
            "canonical CSV digest drifted; update baseline if intentional"
        );
    }

    #[test]
    fn validate_targets_enforce_deep_crossing_failure_cap() {
        let (mut aggregates, records) = satisfied_data();
        let scenario = format!(
            "{} - {}",
            mode_label(GameMode::Deep),
            GameplayStrategy::Conservative
        );
        let target = find_mut_aggregate(&mut aggregates, &scenario);
        target.crossing_failure_rate = 0.25;
        let err = validate_playability_targets(&aggregates, &records).unwrap_err();
        assert!(
            err.to_string().contains("terminal crossing rate"),
            "unexpected error message: {err}"
        );
        assert!(
            err.to_string().contains(&scenario),
            "scenario name missing from error: {err}"
        );
        assert!(
            err.to_string().contains("16%"),
            "deep cap not referenced in error: {err}"
        );
    }

    const TEST_SCENARIOS: &[(GameMode, GameplayStrategy)] = &[
        (GameMode::Classic, GameplayStrategy::Balanced),
        (GameMode::Classic, GameplayStrategy::ResourceManager),
        (GameMode::Deep, GameplayStrategy::Balanced),
        (GameMode::Deep, GameplayStrategy::Conservative),
        (GameMode::Deep, GameplayStrategy::Aggressive),
    ];

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
        metrics.travel_ratio = 0.95;
        metrics.unique_per_20_days = if mode.is_deep() { 1.7 } else { 2.1 };
        metrics.reached_2000_by_day150 = true;
        metrics.crossing_events.clear();
        metrics.crossing_permit_uses = 0;
        metrics.crossing_bribe_attempts = 0;
        metrics.crossing_bribe_successes = 0;
        metrics.crossing_detours_taken = 0;
        metrics.crossing_failures = 0;
        metrics.survived_run = true;
        metrics.failure_family = Some(FailureFamily::Vehicle);
        metrics
    }

    fn base_aggregate(mode: GameMode, strategy: GameplayStrategy) -> PlayabilityAggregate {
        let scenario_name = format!("{} - {}", mode_label(mode), strategy);
        let (
            mean_unique,
            min_unique,
            travel_ratio,
            min_travel,
            pct_2k,
            pants_fail,
            boss_reach,
            boss_win,
        ) = match (mode, strategy) {
            (GameMode::Classic, GameplayStrategy::Balanced) => {
                (2.2, 2.05, 0.95, 0.93, 0.40, 0.15, 0.40, 0.25)
            }
            (GameMode::Classic, GameplayStrategy::ResourceManager) => {
                (2.1, 2.0, 0.94, 0.92, 0.80, 0.20, 0.60, 0.32)
            }
            (GameMode::Deep, GameplayStrategy::Balanced) => {
                (1.7, 1.55, 0.94, 0.93, 0.35, 0.18, 0.55, 0.18)
            }
            (GameMode::Deep, GameplayStrategy::Conservative) => {
                (1.6, 1.5, 0.93, 0.92, 0.30, 0.20, 0.55, 0.12)
            }
            (GameMode::Deep, GameplayStrategy::Aggressive) => {
                (1.6, 1.5, 0.93, 0.92, 0.75, 0.22, 0.70, 0.12)
            }
            _ => (1.6, 1.5, 0.93, 0.92, 0.35, 0.2, 0.65, 0.05),
        };
        PlayabilityAggregate {
            scenario_name,
            mode,
            strategy,
            iterations: 100,
            mean_days: 120.0,
            std_days: 3.0,
            mean_miles: 2_000.0,
            std_miles: 25.0,
            mean_avg_mpd: 15.0,
            boss_reach_pct: boss_reach,
            boss_win_pct: boss_win,
            pants_failure_pct: pants_fail,
            mean_travel_ratio: travel_ratio,
            mean_unique_per_20: mean_unique,
            mean_rotation_events: 5.0,
            pct_reached_2k_by_150: pct_2k,
            min_unique_per_20: min_unique,
            min_travel_ratio: min_travel,
            mean_crossing_events: 1.0,
            crossing_permit_rate: 0.25,
            mean_crossing_bribes: 0.0,
            crossing_bribe_success_rate: 0.0,
            mean_crossing_detours: 0.2,
            crossing_failure_rate: 0.05,
            mean_stop_cap_conversions: 0.1,
            endgame_activation_rate: if mode.is_deep() { 0.75 } else { 0.0 },
            endgame_field_repair_rate: 0.2,
            mean_endgame_cooldown: 1.0,
            survival_rate: 0.70,
            failure_vehicle_pct: 0.25,
            failure_sanity_pct: 0.25,
            failure_exposure_pct: 0.25,
            failure_crossing_pct: 0.25,
        }
    }

    fn find_mut_aggregate<'a>(
        aggregates: &'a mut [PlayabilityAggregate],
        scenario_name: &str,
    ) -> &'a mut PlayabilityAggregate {
        aggregates
            .iter_mut()
            .find(|agg| agg.scenario_name == scenario_name)
            .unwrap_or_else(|| panic!("missing scenario {scenario_name}"))
    }

    #[test]
    fn deterministic_journey_digest_baseline() {
        let digest = journey_digest(0xDEAD_BEEF);
        assert_eq!(
            digest, JOURNEY_LEDGER_DIGEST,
            "journey ledger digest drifted; update baseline if intentional"
        );
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
        use dystrail_game::data::EncounterData;
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

#[derive(Debug, Clone)]
struct AggregateBuilder {
    scenario_name: String,
    mode: GameMode,
    strategy: GameplayStrategy,
    stats_days: RunningStats,
    stats_miles: RunningStats,
    avg_mpd_sum: f64,
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
    stop_cap_sum: u32,
    endgame_active_runs: u32,
    endgame_field_repair_runs: u32,
    endgame_cooldown_sum: u32,
    survival_sum: u32,
    failure_vehicle: u32,
    failure_sanity: u32,
    failure_exposure: u32,
    failure_crossing: u32,
}

impl AggregateBuilder {
    fn new(record: &PlayabilityRecord) -> Self {
        Self {
            scenario_name: record.scenario_name.clone(),
            mode: record.mode,
            strategy: record.strategy,
            stats_days: RunningStats::default(),
            stats_miles: RunningStats::default(),
            avg_mpd_sum: 0.0,
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
            stop_cap_sum: 0,
            endgame_active_runs: 0,
            endgame_field_repair_runs: 0,
            endgame_cooldown_sum: 0,
            survival_sum: 0,
            failure_vehicle: 0,
            failure_sanity: 0,
            failure_exposure: 0,
            failure_crossing: 0,
        }
    }

    fn ingest(&mut self, metrics: &PlayabilityMetrics) {
        self.iterations += 1;
        self.stats_days.add(f64::from(metrics.days_survived));
        self.stats_miles.add(f64::from(metrics.miles_traveled));
        self.avg_mpd_sum += metrics.avg_miles_per_day;
        if metrics.boss_reached {
            self.boss_reached += 1;
        }
        if metrics.boss_won {
            self.boss_won += 1;
        }
        if metrics.final_pants >= 100 || metrics.ending_type.contains("Pants") {
            self.pants_failures += 1;
        }
        if metrics.reached_2000_by_day150 {
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
        self.stop_cap_sum = self
            .stop_cap_sum
            .saturating_add(metrics.stop_cap_conversions);
        if metrics.endgame_active {
            self.endgame_active_runs = self.endgame_active_runs.saturating_add(1);
        }
        if metrics.endgame_field_repair_used {
            self.endgame_field_repair_runs = self.endgame_field_repair_runs.saturating_add(1);
        }
        self.endgame_cooldown_sum = self
            .endgame_cooldown_sum
            .saturating_add(metrics.endgame_cooldown_days);
        if metrics.survived_run {
            self.survival_sum = self.survival_sum.saturating_add(1);
        }
        match metrics.failure_family {
            Some(FailureFamily::Vehicle) => {
                self.failure_vehicle = self.failure_vehicle.saturating_add(1);
            }
            Some(FailureFamily::Sanity) => {
                self.failure_sanity = self.failure_sanity.saturating_add(1);
            }
            Some(FailureFamily::Exposure) => {
                self.failure_exposure = self.failure_exposure.saturating_add(1);
            }
            Some(FailureFamily::Crossing) => {
                self.failure_crossing = self.failure_crossing.saturating_add(1);
            }
            _ => {}
        }
    }

    fn finish(self) -> PlayabilityAggregate {
        let iterations_u32 = self.iterations.max(1);
        let iterations = usize::try_from(self.iterations).unwrap_or(usize::MAX);
        let denom = f64::from(iterations_u32);
        let failure_total = f64::from(
            self.failure_vehicle
                + self.failure_sanity
                + self.failure_exposure
                + self.failure_crossing,
        );
        let failure_pct = |count: u32| -> f64 {
            if failure_total <= f64::EPSILON {
                0.0
            } else {
                f64::from(count) / failure_total
            }
        };
        PlayabilityAggregate {
            scenario_name: self.scenario_name,
            mode: self.mode,
            strategy: self.strategy,
            iterations,
            mean_days: self.stats_days.mean(),
            std_days: self.stats_days.std_dev(),
            mean_miles: self.stats_miles.mean(),
            std_miles: self.stats_miles.std_dev(),
            mean_avg_mpd: self.avg_mpd_sum / denom,
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
            mean_stop_cap_conversions: f64::from(self.stop_cap_sum) / denom,
            endgame_activation_rate: f64::from(self.endgame_active_runs) / denom,
            endgame_field_repair_rate: f64::from(self.endgame_field_repair_runs) / denom,
            mean_endgame_cooldown: f64::from(self.endgame_cooldown_sum) / denom,
            survival_rate: f64::from(self.survival_sum) / denom,
            failure_vehicle_pct: failure_pct(self.failure_vehicle),
            failure_sanity_pct: failure_pct(self.failure_sanity),
            failure_exposure_pct: failure_pct(self.failure_exposure),
            failure_crossing_pct: failure_pct(self.failure_crossing),
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
