use std::fmt;

use dystrail_game::GameState;
use dystrail_game::data::{Choice, Encounter};

/// Decision returned by a [`PlayerPolicy`]
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub choice_index: usize,
    pub rationale: Option<String>,
}

impl PolicyDecision {
    #[must_use]
    pub const fn new(choice_index: usize, rationale: Option<String>) -> Self {
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
}

impl GameplayStrategy {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Conservative => "Conservative",
            Self::Aggressive => "Aggressive",
            Self::Balanced => "Balanced",
            Self::ResourceManager => "Resource Manager",
        }
    }

    #[must_use]
    pub fn create_policy(self, _seed: u64) -> Box<dyn PlayerPolicy + Send> {
        match self {
            Self::Conservative => Box::new(ConservativePolicy),
            Self::Aggressive => Box::new(AggressivePolicy),
            Self::Balanced => Box::new(BalancedPolicy),
            Self::ResourceManager => Box::new(ResourceManagerPolicy),
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
            .map(|(idx, choice)| (idx, aggressive_score(choice)))
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

const fn conservative_risk(choice: &Choice) -> i32 {
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

fn aggressive_score(choice: &Choice) -> i32 {
    aggressive_reward(choice) + choice.effects.pants.max(0) * 2
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

#[cfg(test)]
mod tests {
    use super::*;
    use dystrail_game::data::{Choice, Effects, Encounter};

    fn sample_encounter() -> Encounter {
        Encounter {
            id: "enc1".into(),
            name: "Encounter".into(),
            desc: "Desc".into(),
            weight: 1,
            regions: vec![],
            modes: vec![],
            choices: vec![
                Choice {
                    label: "Risky".into(),
                    effects: Effects {
                        hp: -2,
                        supplies: 0,
                        sanity: -1,
                        pants: 0,
                        ..Effects::default()
                    },
                },
                Choice {
                    label: "Reward".into(),
                    effects: Effects {
                        hp: 1,
                        supplies: 2,
                        credibility: 1,
                        ..Effects::default()
                    },
                },
            ],
            hard_stop: false,
            major_repair: false,
            chainable: false,
        }
    }

    #[test]
    fn strategy_labels_are_stable() {
        assert_eq!(GameplayStrategy::Balanced.label(), "Balanced");
        assert_eq!(
            GameplayStrategy::ResourceManager.label(),
            "Resource Manager"
        );
    }

    #[test]
    fn conservative_prefers_lower_risk() {
        let encounter = sample_encounter();
        let mut policy = ConservativePolicy;
        let decision = policy.pick_choice(&GameState::default(), &encounter);
        assert_eq!(decision.choice_index, 1);
    }

    #[test]
    fn aggressive_prefers_higher_reward() {
        let encounter = sample_encounter();
        let mut policy = AggressivePolicy;
        let decision = policy.pick_choice(&GameState::default(), &encounter);
        assert_eq!(decision.choice_index, 1);
    }

    #[test]
    fn balanced_scores_combine_reward_and_risk() {
        let encounter = sample_encounter();
        let mut policy = BalancedPolicy;
        let decision = policy.pick_choice(&GameState::default(), &encounter);
        assert_eq!(decision.choice_index, 1);
    }

    #[test]
    fn resource_manager_prefers_penalty_minimization() {
        let encounter = sample_encounter();
        let mut policy = ResourceManagerPolicy;
        let decision = policy.pick_choice(&GameState::default(), &encounter);
        assert_eq!(decision.choice_index, 1);
    }
}
