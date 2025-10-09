use crate::logic::game_tester::{GameTester, GameplayStrategy, PlayabilityMetrics};
use crate::logic::seeds::SeedInfo;
use dystrail_game::GameMode;

#[derive(Debug, Clone)]
pub struct PlayabilityRecord {
    pub scenario_name: String,
    pub mode: GameMode,
    pub strategy: GameplayStrategy,
    pub seed_code: String,
    pub seed_value: u64,
    pub metrics: PlayabilityMetrics,
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

pub fn run_playability_analysis(seeds: &[SeedInfo], verbose: bool) -> Vec<PlayabilityRecord> {
    let tester = GameTester::new(verbose);
    let mut records = Vec::with_capacity(seeds.len() * PLAYABILITY_SCENARIOS.len());

    for &(mode, strategy) in PLAYABILITY_SCENARIOS {
        for seed in seeds.iter().filter(|seed| seed.matches_mode(mode)) {
            let metrics = tester.play_game(mode, strategy, seed.seed);
            let scenario_name = format!("{} - {}", mode_label(mode), strategy);
            let seed_code = seed.share_code_for_mode(mode);

            records.push(PlayabilityRecord {
                scenario_name,
                mode,
                strategy,
                seed_code,
                seed_value: seed.seed,
                metrics,
            });
        }
    }

    records
}

fn mode_label(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Classic => "Classic",
        GameMode::Deep => "Deep",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::seeds::SeedInfo;

    #[test]
    fn generates_records_for_each_scenario() {
        let seeds = vec![SeedInfo::from_numeric(1337)];
        let records = run_playability_analysis(&seeds, false);
        assert_eq!(records.len(), PLAYABILITY_SCENARIOS.len());
        assert!(records.iter().all(|r| !r.seed_code.is_empty()));
    }
}
