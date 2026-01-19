use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::endgame::EndgameTravelCfg;
use crate::game::state::{GameMode, GameState};
use crate::game::weather::WeatherConfig;
use crate::game::{JourneySession, MechanicalPolicyId, StrategyId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Boot,
    Persona,
    Outfitting,
    Menu,
    Travel,
    Store,
    Crossing,
    RoutePrompt,
    Camp,
    Encounter,
    Boss,
    Result,
}

#[must_use]
pub fn is_seed_code_valid(code: &str) -> bool {
    regex::Regex::new(r"^(CL|DP)-[A-Z0-9]+\d{2}$")
        .map(|re| re.is_match(code))
        .unwrap_or(false)
}

const fn default_strategy_for(mode: GameMode) -> StrategyId {
    match mode {
        GameMode::Classic | GameMode::Deep => StrategyId::Balanced,
    }
}

#[must_use]
pub fn strategy_for_state(state: &GameState) -> StrategyId {
    state
        .policy
        .map_or_else(|| default_strategy_for(state.mode), StrategyId::from)
}

#[must_use]
pub fn session_from_state(state: GameState, endgame_cfg: &EndgameTravelCfg) -> JourneySession {
    let strategy = strategy_for_state(&state);
    JourneySession::from_state(state, strategy, endgame_cfg)
}

#[must_use]
pub fn phase_for_state(state: &GameState) -> Phase {
    let boss_gate = state.mechanical_policy == MechanicalPolicyId::DystrailLegacy
        && state.boss.readiness.ready
        && !state.boss.outcome.attempted;
    let dystrail_crossing = state.mechanical_policy == MechanicalPolicyId::DystrailLegacy
        && state.pending_crossing.is_some();
    let otdeluxe_crossing = state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s
        && state.ot_deluxe.crossing.choice_pending;
    let otdeluxe_store = state.mechanical_policy == MechanicalPolicyId::OtDeluxe90s
        && state.ot_deluxe.store.pending_node.is_some();

    if state.ending.is_some() || state.stats.pants >= 100 {
        Phase::Result
    } else if state.ot_deluxe.route.pending_prompt.is_some() {
        Phase::RoutePrompt
    } else if otdeluxe_crossing || dystrail_crossing {
        Phase::Crossing
    } else if otdeluxe_store {
        Phase::Store
    } else if state.current_encounter.is_some() {
        Phase::Encounter
    } else if boss_gate {
        Phase::Boss
    } else {
        Phase::Travel
    }
}

#[must_use]
pub fn build_weather_badge(state: &GameState, cfg: &WeatherConfig) -> WeatherBadge {
    let weather_today = state.weather_state.today;
    let mitigated = cfg
        .mitigation
        .get(&weather_today)
        .is_some_and(|mit| state.inventory.tags.contains(&mit.tag));
    WeatherBadge {
        weather: weather_today,
        mitigated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::CrossingKind;
    use crate::game::data::{Choice, Effects, Encounter};
    use crate::game::exec_orders::ExecOrder;
    use crate::game::state::{Ending, PendingCrossing, PolicyKind};

    fn encounter_stub() -> Encounter {
        Encounter {
            id: String::from("enc"),
            name: String::from("Encounter"),
            desc: String::new(),
            weight: 1,
            regions: Vec::new(),
            modes: Vec::new(),
            choices: vec![Choice {
                label: String::from("Continue"),
                effects: Effects::default(),
            }],
            hard_stop: false,
            major_repair: false,
            chainable: false,
        }
    }

    #[test]
    fn strategy_and_phase_helpers_cover_branches() {
        let mut state = GameState::default();
        assert_eq!(strategy_for_state(&state), StrategyId::Balanced);
        state.policy = Some(PolicyKind::Aggressive);
        assert_eq!(strategy_for_state(&state), StrategyId::Aggressive);

        let endgame_cfg = EndgameTravelCfg::default_config();
        let session = session_from_state(state.clone(), &endgame_cfg);
        assert_eq!(session.state().seed, state.seed);

        let mut state = GameState::default();
        assert_eq!(phase_for_state(&state), Phase::Travel);

        state.current_encounter = Some(encounter_stub());
        assert_eq!(phase_for_state(&state), Phase::Encounter);
        state.current_encounter = None;

        state.pending_crossing = Some(PendingCrossing {
            kind: CrossingKind::Checkpoint,
            computed_miles_today: 0.0,
        });
        assert_eq!(phase_for_state(&state), Phase::Crossing);
        state.pending_crossing = None;

        state.mechanical_policy = MechanicalPolicyId::OtDeluxe90s;
        state.ot_deluxe.store.pending_node = Some(3);
        assert_eq!(phase_for_state(&state), Phase::Store);
        state.ot_deluxe.store.pending_node = None;

        state.ot_deluxe.crossing.choice_pending = true;
        assert_eq!(phase_for_state(&state), Phase::Crossing);
        state.ot_deluxe.crossing.choice_pending = false;

        state.ot_deluxe.route.pending_prompt =
            Some(crate::game::OtDeluxeRoutePrompt::SubletteCutoff);
        assert_eq!(phase_for_state(&state), Phase::RoutePrompt);
        state.ot_deluxe.route.pending_prompt = None;

        state.mechanical_policy = MechanicalPolicyId::DystrailLegacy;
        state.boss.readiness.ready = true;
        state.boss.outcome.attempted = false;
        assert_eq!(phase_for_state(&state), Phase::Boss);

        state.ending = Some(Ending::BossVictory);
        assert_eq!(phase_for_state(&state), Phase::Result);
        state.ending = None;
        state.stats.pants = 120;
        assert_eq!(phase_for_state(&state), Phase::Result);

        state.current_order = Some(ExecOrder::Shutdown);
        let _ = state.current_order;
    }

    #[test]
    fn build_weather_badge_marks_mitigation() {
        let mut state = GameState::default();
        let config = WeatherConfig::load_from_static();
        if let Some((weather, mitigation)) = config.mitigation.iter().next() {
            state.weather_state.today = *weather;
            state.inventory.tags.insert(mitigation.tag.clone());
            let badge = build_weather_badge(&state, &config);
            assert!(badge.mitigated);
        } else {
            let badge = build_weather_badge(&state, &config);
            assert!(!badge.mitigated);
        }
    }
}
