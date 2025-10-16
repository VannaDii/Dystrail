use std::fmt;

use dystrail_game::GameState;
use dystrail_game::data::{Choice, Encounter};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Decision returned by a [`PlayerPolicy`]
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub choice_index: usize,
    pub rationale: Option<String>,
}

impl PolicyDecision {
    #[must_use]
    pub fn new(choice_index: usize, rationale: Option<String>) -> Self {
        Self {
            choice_index,
            rationale,
        }
    }
}

/// Policy interface for automated play strategies.
pub trait PlayerPolicy {
    /// Name used for logging/debug output.
    fn name(&self) -> &'static str;

    /// Select a choice for an active encounter.
    fn pick_choice(&mut self, state: &GameState, encounter: &Encounter) -> PolicyDecision;
}

/// Built-in gameplay strategies for automated runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameplayStrategy {
    Conservative,
    Aggressive,
    Balanced,
    ResourceManager,
    MonteCarlo,
}

impl GameplayStrategy {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            GameplayStrategy::Conservative => "Conservative",
            GameplayStrategy::Aggressive => "Aggressive",
            GameplayStrategy::Balanced => "Balanced",
            GameplayStrategy::ResourceManager => "Resource Manager",
            GameplayStrategy::MonteCarlo => "Monte Carlo",
        }
    }

    #[must_use]
    pub fn create_policy(self, seed: u64) -> Box<dyn PlayerPolicy + Send> {
        match self {
            GameplayStrategy::Conservative => Box::new(ConservativePolicy),
            GameplayStrategy::Aggressive => Box::new(AggressivePolicy),
            GameplayStrategy::Balanced => Box::new(BalancedPolicy),
            GameplayStrategy::ResourceManager => Box::new(ResourceManagerPolicy),
            GameplayStrategy::MonteCarlo => Box::new(MonteCarloPolicy::new(seed)),
        }
    }
}

impl fmt::Display for GameplayStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

struct ConservativePolicy;
struct AggressivePolicy;
struct BalancedPolicy;
struct ResourceManagerPolicy;

struct MonteCarloPolicy {
    rng: ChaCha20Rng,
    simulations: u32,
}

impl MonteCarloPolicy {
    fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha20Rng::seed_from_u64(seed),
            simulations: 12,
        }
    }
}

impl PlayerPolicy for ConservativePolicy {
    fn name(&self) -> &'static str {
        "Conservative"
    }

    fn pick_choice(&mut self, _state: &GameState, encounter: &Encounter) -> PolicyDecision {
        let (idx, risk) = encounter
            .choices
            .iter()
            .enumerate()
            .map(|(idx, choice)| (idx, conservative_risk(choice)))
            .min_by_key(|(_, risk)| *risk)
            .unwrap_or((0, 0));

        PolicyDecision::new(
            Some(idx)
                .filter(|_| !encounter.choices.is_empty())
                .unwrap_or(0),
            Some(format!("risk {risk}")),
        )
    }
}

impl PlayerPolicy for AggressivePolicy {
    fn name(&self) -> &'static str {
        "Aggressive"
    }

    fn pick_choice(&mut self, _state: &GameState, encounter: &Encounter) -> PolicyDecision {
        let (idx, reward) = encounter
            .choices
            .iter()
            .enumerate()
            .map(|(idx, choice)| (idx, aggressive_reward(choice)))
            .max_by_key(|(_, reward)| *reward)
            .unwrap_or((0, 0));

        PolicyDecision::new(
            Some(idx)
                .filter(|_| !encounter.choices.is_empty())
                .unwrap_or(0),
            Some(format!("reward {reward}")),
        )
    }
}

impl PlayerPolicy for BalancedPolicy {
    fn name(&self) -> &'static str {
        "Balanced"
    }

    fn pick_choice(&mut self, _state: &GameState, encounter: &Encounter) -> PolicyDecision {
        let (idx, score) = encounter
            .choices
            .iter()
            .enumerate()
            .map(|(idx, choice)| (idx, balanced_score(choice)))
            .max_by_key(|(_, score)| *score)
            .unwrap_or((0, 0));

        PolicyDecision::new(
            Some(idx)
                .filter(|_| !encounter.choices.is_empty())
                .unwrap_or(0),
            Some(format!("score {score}")),
        )
    }
}

impl PlayerPolicy for ResourceManagerPolicy {
    fn name(&self) -> &'static str {
        "Resource Manager"
    }

    fn pick_choice(&mut self, _state: &GameState, encounter: &Encounter) -> PolicyDecision {
        let (idx, penalty) = encounter
            .choices
            .iter()
            .enumerate()
            .map(|(idx, choice)| (idx, resource_penalty(choice)))
            .min_by_key(|(_, penalty)| *penalty)
            .unwrap_or((0, 0));

        PolicyDecision::new(
            Some(idx)
                .filter(|_| !encounter.choices.is_empty())
                .unwrap_or(0),
            Some(format!("penalty {penalty}")),
        )
    }
}

impl PlayerPolicy for MonteCarloPolicy {
    fn name(&self) -> &'static str {
        "Monte Carlo"
    }

    fn pick_choice(&mut self, state: &GameState, encounter: &Encounter) -> PolicyDecision {
        if encounter.choices.is_empty() {
            return PolicyDecision::new(0, Some("no choices".to_string()));
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut best_idx = 0;

        for (idx, choice) in encounter.choices.iter().enumerate() {
            let score = simulate_choice_outcome(state, choice, &mut self.rng, self.simulations);
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        PolicyDecision::new(best_idx, Some(format!("score {best_score:.2}")))
    }
}

fn conservative_risk(choice: &Choice) -> i32 {
    let eff = &choice.effects;
    let mut risk = 0;
    if eff.hp < 0 {
        risk += (-eff.hp) * 4;
    }
    if eff.supplies < 0 {
        risk += (-eff.supplies) * 3;
    }
    if eff.sanity < 0 {
        risk += (-eff.sanity) * 2;
    }
    if eff.pants > 0 {
        risk += eff.pants * 2;
    }
    risk
}

fn aggressive_reward(choice: &Choice) -> i32 {
    let eff = &choice.effects;
    let mut reward = 0;
    reward += eff.hp.max(0) * 2;
    reward += eff.supplies.max(0) * 2;
    reward += eff.credibility.max(0) * 3;
    reward += eff.allies.max(0);
    reward -= eff.pants.max(0);
    reward
}

fn balanced_score(choice: &Choice) -> i32 {
    aggressive_reward(choice) - conservative_risk(choice)
}

fn resource_penalty(choice: &Choice) -> i32 {
    let eff = &choice.effects;
    let mut penalty = (-eff.hp).max(0) * 6
        + (-eff.supplies).max(0) * 4
        + (-eff.sanity).max(0) * 3
        + eff.pants.max(0) * 5;
    if eff.pants < 0 {
        penalty -= (-eff.pants) * 3;
    }
    if eff.supplies > 0 {
        penalty -= eff.supplies * 2;
    }
    if eff.hp > 0 {
        penalty -= eff.hp * 2;
    }
    penalty
}

fn simulate_choice_outcome(
    state: &GameState,
    choice: &Choice,
    rng: &mut ChaCha20Rng,
    simulations: u32,
) -> f64 {
    let iterations = simulations.max(1);
    let mut total = 0.0_f64;
    for _ in 0..iterations {
        let mut score = f64::from(balanced_score(choice));
        let eff = &choice.effects;
        let projected_hp = state.stats.hp + eff.hp;
        let projected_sanity = state.stats.sanity + eff.sanity;
        let projected_supplies = state.stats.supplies + eff.supplies;
        let projected_pants = state.stats.pants + eff.pants;

        score -= deficiency_penalty(projected_hp, 3) * 4.0;
        score -= deficiency_penalty(projected_sanity, 3) * 3.0;
        score -= deficiency_penalty(projected_supplies, 3) * 5.0;

        if projected_pants > 60 {
            score -= f64::from(projected_pants - 60) * 1.5;
        }

        score += rng.random::<f64>();
        total += score;
    }
    total / f64::from(iterations)
}

fn deficiency_penalty(value: i32, floor: i32) -> f64 {
    if value >= floor {
        0.0
    } else {
        f64::from(floor - value)
    }
}
