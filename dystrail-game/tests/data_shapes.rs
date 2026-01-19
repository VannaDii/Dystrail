use std::collections::BTreeMap;
use std::hash::Hasher;

use dystrail_game::data::EncounterData;
use dystrail_game::{
    GameMode, GameState, JourneyController, MechanicalPolicyId, PolicyId, StrategyId, TravelDayKind,
};
use serde_json::{Map, Value};
use twox_hash::XxHash64;

const SNAPSHOT_HASH: u64 = 0xa8b5_b3de_3ffc_6c5b;

#[test]
fn journey_config_snapshot_stable() {
    let combos = [
        (PolicyId::Classic, StrategyId::Balanced),
        (PolicyId::Classic, StrategyId::Aggressive),
        (PolicyId::Classic, StrategyId::Conservative),
        (PolicyId::Classic, StrategyId::ResourceManager),
        (PolicyId::Deep, StrategyId::Balanced),
        (PolicyId::Deep, StrategyId::Aggressive),
        (PolicyId::Deep, StrategyId::Conservative),
        (PolicyId::Deep, StrategyId::ResourceManager),
    ];

    let mut snapshot = BTreeMap::new();
    for (policy, strategy) in combos {
        let controller = JourneyController::new(
            MechanicalPolicyId::DystrailLegacy,
            policy,
            strategy,
            0x00C0_FFEE,
        );
        let value = canonicalize_value(serde_json::to_value(controller.config()).unwrap());
        let key = format!("{}:{}", policy_label(policy), strategy_label(strategy));
        snapshot.insert(key, value);
    }
    let canonical = serde_json::to_string_pretty(&snapshot).unwrap();
    let digest = snapshot_hash(canonical.as_bytes());
    assert_eq!(
        digest, SNAPSHOT_HASH,
        "journey config snapshot changed\n{canonical}"
    );
}

#[test]
fn game_state_serialization_preserves_day_records() {
    let encounters = EncounterData::from_json(include_str!(
        "../../dystrail-web/static/assets/data/game.json"
    ))
    .unwrap();
    let mut state = GameState::default().with_seed(0xFACE_B00C, GameMode::Classic, encounters);
    let mut controller = JourneyController::new(
        MechanicalPolicyId::DystrailLegacy,
        PolicyId::Classic,
        StrategyId::Balanced,
        0xFACE_B00C,
    );
    controller.configure_state(&mut state);
    for _ in 0..3 {
        let outcome = controller.tick_day(&mut state);
        assert!(outcome.record.is_some(), "expected day record");
        if outcome.ended {
            break;
        }
        state.day = state.day.saturating_add(1);
        state.current_day_record = None;
        state.current_day_kind = None;
    }
    assert!(
        !state.day_records.is_empty(),
        "simulation should record travel days"
    );

    let saved = serde_json::to_string(&state).unwrap();
    let restored: GameState = serde_json::from_str(&saved).unwrap();

    let original_value = serde_json::to_value(&state).unwrap();
    let restored_value = serde_json::to_value(&restored).unwrap();
    assert_eq!(original_value, restored_value, "round-trip mismatch");
    assert_eq!(restored.day_records, state.day_records);
    assert_eq!(restored.endgame.active, state.endgame.active);
    // ensure ledger recompute still works
    let (kind, _) = dystrail_game::day_accounting::record_travel_day(
        &mut state.clone(),
        TravelDayKind::Travel,
        10.0,
    );
    assert!(kind.counts_toward_ratio());
}

fn canonicalize_value(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(canonicalize_value)
                .collect::<Vec<_>>(),
        ),
        Value::Object(map) => {
            let mut result = Map::with_capacity(map.len());
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            for (key, value) in entries {
                result.insert(key, canonicalize_value(value));
            }
            Value::Object(result)
        }
        other => other,
    }
}

const fn policy_label(policy: PolicyId) -> &'static str {
    match policy {
        PolicyId::Classic => "classic",
        PolicyId::Deep => "deep",
    }
}

const fn strategy_label(strategy: StrategyId) -> &'static str {
    match strategy {
        StrategyId::Balanced => "balanced",
        StrategyId::Aggressive => "aggressive",
        StrategyId::Conservative => "conservative",
        StrategyId::ResourceManager => "resource_manager",
    }
}

fn snapshot_hash(bytes: &[u8]) -> u64 {
    let mut hasher = XxHash64::with_seed(0);
    hasher.write(bytes);
    hasher.finish()
}
