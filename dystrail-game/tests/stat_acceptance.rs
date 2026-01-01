use dystrail_game::crossings::{CrossingContext, CrossingResult};
use dystrail_game::journey::JourneyCfg;
use dystrail_game::state::{GameMode, GameState, PaceId, PolicyKind};
use dystrail_game::{CrossingPolicy, JourneyController, MechanicalPolicyId, PolicyId, StrategyId};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::convert::TryFrom;

const SAMPLE_SIZE: usize = 5000;
const TOLERANCE: f64 = 0.025;

#[test]
fn breakdown_probability_tracks_base_rate() {
    let mut state = GameState {
        mode: GameMode::Classic,
        policy: Some(PolicyKind::Balanced),
        ..GameState::default()
    };
    state.journey_breakdown.base = 0.12;
    state.journey_breakdown.beta = 0.0;
    state.journey_breakdown.pace_factor = std::iter::once((PaceId::Steady, 1.0)).collect();
    state.journey_breakdown.weather_factor =
        std::iter::once((dystrail_game::weather::Weather::Clear, 1.0)).collect();
    state.pace = PaceId::Steady;
    state.weather_state.today = dystrail_game::weather::Weather::Clear;
    state.vehicle.set_wear(0.0);
    state.attach_rng_bundle(std::rc::Rc::new(
        dystrail_game::journey::RngBundle::from_user_seed(1234),
    ));

    let sample_size = u32::try_from(SAMPLE_SIZE).expect("sample size fits u32");
    let mut triggered = 0usize;
    for _ in 0..SAMPLE_SIZE {
        let rolled = state.vehicle_roll_for_testing();
        if rolled {
            triggered += 1;
            state.breakdown = None;
        }
    }
    let observed =
        f64::from(u32::try_from(triggered).expect("count fits")) / f64::from(sample_size);
    assert!(
        (observed - 0.12).abs() <= TOLERANCE,
        "breakdown rate drifted: observed {observed:.4}"
    );
}

#[test]
fn crossing_distribution_matches_policy_weights() {
    let mut policy = CrossingPolicy {
        pass: 0.65,
        detour: 0.25,
        terminal: 0.10,
        ..CrossingPolicy::default()
    };
    policy.sanitize();
    let ctx = CrossingContext {
        policy: &policy,
        kind: dystrail_game::crossings::CrossingKind::Checkpoint,
        has_permit: false,
        bribe_intent: false,
        prior_bribe_attempts: 0,
    };
    let mut rng = SmallRng::seed_from_u64(0xACED);

    let mut pass = 0usize;
    let mut detour = 0usize;
    let mut terminal = 0usize;
    for _ in 0..SAMPLE_SIZE {
        match dystrail_game::crossings::resolve_crossing(ctx, &mut rng).result {
            CrossingResult::Pass => pass += 1,
            CrossingResult::Detour(_) => detour += 1,
            CrossingResult::TerminalFail => terminal += 1,
        }
    }
    let total = f64::from(u32::try_from(SAMPLE_SIZE).expect("sample size fits"));
    let pass_rate = f64::from(u32::try_from(pass).expect("count fits")) / total;
    let detour_rate = f64::from(u32::try_from(detour).expect("count fits")) / total;
    let terminal_rate = f64::from(u32::try_from(terminal).expect("count fits")) / total;
    assert!((pass_rate - 0.65).abs() <= TOLERANCE);
    assert!((detour_rate - 0.25).abs() <= TOLERANCE);
    assert!((terminal_rate - 0.10).abs() <= TOLERANCE);
}

#[test]
fn bribe_increases_pass_and_reduces_terminal() {
    let mut policy = CrossingPolicy {
        pass: 0.5,
        detour: 0.3,
        terminal: 0.2,
        bribe: dystrail_game::journey::BribePolicy {
            pass_bonus: 0.2,
            detour_bonus: 0.0,
            terminal_penalty: 0.2,
            diminishing_returns: 0.0,
        },
        ..CrossingPolicy::default()
    };
    policy.sanitize();
    let base_ctx = CrossingContext {
        policy: &policy,
        kind: dystrail_game::crossings::CrossingKind::Checkpoint,
        has_permit: false,
        bribe_intent: false,
        prior_bribe_attempts: 0,
    };
    let bribe_ctx = CrossingContext {
        bribe_intent: true,
        ..base_ctx
    };
    let mut rng = SmallRng::seed_from_u64(0xACED_F00D);

    let mut base_pass = 0u32;
    let mut base_terminal = 0u32;
    for _ in 0..SAMPLE_SIZE {
        match dystrail_game::crossings::resolve_crossing(base_ctx, &mut rng).result {
            CrossingResult::Pass => base_pass += 1,
            CrossingResult::TerminalFail => base_terminal += 1,
            CrossingResult::Detour(_) => {}
        }
    }

    let mut bribe_pass = 0u32;
    let mut bribe_terminal = 0u32;
    let mut rng_bribe = SmallRng::seed_from_u64(0xACED_F00D);
    for _ in 0..SAMPLE_SIZE {
        match dystrail_game::crossings::resolve_crossing(bribe_ctx, &mut rng_bribe).result {
            CrossingResult::Pass => bribe_pass += 1,
            CrossingResult::TerminalFail => bribe_terminal += 1,
            CrossingResult::Detour(_) => {}
        }
    }

    assert!(
        bribe_pass > base_pass,
        "bribe should improve pass count (base {base_pass}, bribe {bribe_pass})"
    );
    assert!(
        bribe_terminal < base_terminal,
        "bribe should reduce terminal count (base {base_terminal}, bribe {bribe_terminal})"
    );
}

#[test]
fn endgame_breakdown_scale_reduces_breaks() {
    let mut cfg = JourneyCfg::default();
    cfg.breakdown.base = 0.2;
    cfg.breakdown.beta = 0.0;
    cfg.breakdown.pace_factor = std::iter::once((PaceId::Steady, 1.0)).collect();
    cfg.breakdown.weather_factor =
        std::iter::once((dystrail_game::weather::Weather::Clear, 1.0)).collect();
    let mut controller = JourneyController::with_config(
        MechanicalPolicyId::DystrailLegacy,
        PolicyId::Deep,
        StrategyId::Balanced,
        cfg,
        77,
        dystrail_game::endgame::EndgameTravelCfg::default_config(),
    );

    let mut state = GameState {
        mode: GameMode::Deep,
        policy: Some(PolicyKind::Balanced),
        ..GameState::default()
    };
    let _ = controller.tick_day(&mut state);
    state.endgame.active = true;
    state.endgame.breakdown_scale = 0.2;
    state.vehicle.set_wear(0.0);
    state.pace = PaceId::Steady;
    state.weather_state.today = dystrail_game::weather::Weather::Clear;
    state.journey_breakdown.base = 0.2;
    state.journey_breakdown.beta = 0.0;
    state.journey_breakdown.pace_factor = std::iter::once((PaceId::Steady, 1.0)).collect();
    state.journey_breakdown.weather_factor =
        std::iter::once((dystrail_game::weather::Weather::Clear, 1.0)).collect();
    state.attach_rng_bundle(std::rc::Rc::new(
        dystrail_game::journey::RngBundle::from_user_seed(55),
    ));

    let sample_size = u32::try_from(SAMPLE_SIZE).expect("sample size fits u32");
    let mut breaks_scaled = 0usize;
    for _ in 0..SAMPLE_SIZE {
        if state.vehicle_roll_for_testing() {
            breaks_scaled += 1;
            state.breakdown = None;
        }
    }
    let scaled_rate =
        f64::from(u32::try_from(breaks_scaled).expect("count fits")) / f64::from(sample_size);

    // Baseline without scale
    state.endgame.breakdown_scale = 1.0;
    state.attach_rng_bundle(std::rc::Rc::new(
        dystrail_game::journey::RngBundle::from_user_seed(55),
    ));
    let mut breaks_baseline = 0usize;
    for _ in 0..SAMPLE_SIZE {
        if state.vehicle_roll_for_testing() {
            breaks_baseline += 1;
            state.breakdown = None;
        }
    }
    let base_rate =
        f64::from(u32::try_from(breaks_baseline).expect("count fits")) / f64::from(sample_size);

    assert!(
        scaled_rate < base_rate * 0.35,
        "scaled breakdown rate should drop significantly (scaled {scaled_rate:.4}, base {base_rate:.4})"
    );
}
