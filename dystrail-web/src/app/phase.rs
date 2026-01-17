use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::endgame::EndgameTravelCfg;
use crate::game::state::{GameMode, GameState};
use crate::game::weather::WeatherConfig;
use crate::game::{JourneySession, MechanicalPolicyId, StrategyId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Boot,
    Persona,
    Outfitting,
    Menu,
    Travel,
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

    if state.ending.is_some() || state.stats.pants >= 100 {
        Phase::Result
    } else if state.ot_deluxe.route.pending_prompt.is_some() {
        Phase::RoutePrompt
    } else if otdeluxe_crossing || dystrail_crossing {
        Phase::Crossing
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
