// Temporary diagnostic appended to playability.rs in an isolated workspace copy.
// Not part of the shipped game or tester. Run as hearing_release_diagnostic.
#[test]
fn hearing_release_diagnostic() {
    let tester = GameTester::new(std::sync::Arc::new(crate::logic::TesterAssets::load_default()), false);
    let plan = full_game_plan(GameMode::Classic, GameplayStrategy::Balanced);
    let mut rows = Vec::new();
    for seed in 1337..2337 {
        let summary = tester.run_plan(&plan, seed);
        rows.push(serde_json::json!({
            "seed": seed,
            "hearing": summary.final_state.boss.hearing,
            "victory": summary.final_state.boss.outcome.victory,
        }));
    }
    std::fs::write("/tmp/dystrail-hearing-arrivals.json", serde_json::to_string_pretty(&rows).unwrap()).unwrap();
}
