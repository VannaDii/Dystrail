use crate::game::state::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossOutcome {
    PassedCloture,
    SurvivedFlood,
    PantsEmergency,
    Exhausted,
}

pub fn run_boss_minigame(state: &mut GameState) -> BossOutcome {
    // Phase 1: Cloture — if credibility >= 10 auto-pass
    if state.stats.credibility >= 10 {
        return BossOutcome::PassedCloture;
    }
    // Otherwise attempt up to 3 holds; each reduces pants by small amount if allies>0
    for _ in 0..3 {
        if state.stats.allies > 0 {
            state.stats.pants = (state.stats.pants - 3).max(0);
            state.stats.allies -= 1;
        }
    }
    // Phase 2/3 simplified survival rounds
    let rounds = 5u8;
    for _ in 0..rounds {
        let roll = state.next_pct();
        if roll < 30 {
            state.stats.pants += 7;
        }
        if roll.is_multiple_of(2) {
            state.stats.sanity -= 1;
        }
        state.stats.clamp();
        if state.stats.pants >= 100 {
            return BossOutcome::PantsEmergency;
        }
    }
    BossOutcome::SurvivedFlood
}
