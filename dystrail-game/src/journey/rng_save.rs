//! Preserve deterministic stream positions across game saves without changing RNG algorithms.
use super::{CountingRng, RngBundle, SmallRng};
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::rc::Rc;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) enum RngCall {
    U32,
    U64,
    Bytes(usize),
}
impl Serialize for CountingRng<SmallRng> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.seed, &self.calls).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CountingRng<SmallRng> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (seed, calls): (u64, Vec<RngCall>) = Deserialize::deserialize(deserializer)?;
        if calls.len() > 1_000_000 {
            return Err(serde::de::Error::custom("RNG history exceeds game limit"));
        }
        let mut restored = Self::new(seed);
        for call in &calls {
            match call {
                RngCall::U32 => {
                    restored.rng.next_u32();
                }
                RngCall::U64 => {
                    restored.rng.next_u64();
                }
                RngCall::Bytes(size) => {
                    if *size > 1_048_576 {
                        return Err(serde::de::Error::custom("RNG byte draw exceeds game limit"));
                    }
                    let mut bytes = vec![0; *size];
                    restored.rng.fill_bytes(&mut bytes);
                }
            }
        }
        restored.draws = u64::try_from(calls.len()).map_err(serde::de::Error::custom)?;
        restored.calls = calls;
        Ok(restored)
    }
}
/// Serialize the current random streams.
///
/// # Errors
/// Returns an error if the serializer cannot write the snapshot.
pub fn serialize<S: Serializer>(
    value: &Option<Rc<RngBundle>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    value.as_deref().serialize(serializer)
}
/// Restore the current random streams.
///
/// # Errors
/// Rejects malformed or oversized draw histories.
pub fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Rc<RngBundle>>, D::Error> {
    Option::<RngBundle>::deserialize(deserializer).map(|value| value.map(Rc::new))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_resumed_journey_matches_an_uninterrupted_journey() {
        let data = crate::EncounterData::empty();
        let cfg = crate::EndgameTravelCfg::default();
        let mut original = super::super::JourneySession::new(
            crate::GameMode::Classic,
            super::super::StrategyId::Balanced,
            4242,
            data.clone(),
            &cfg,
        );
        for _ in 0..3 {
            original.tick_day();
        }
        let encoded = serde_json::to_string(original.state()).expect("save");
        let restored: crate::GameState = serde_json::from_str(&encoded).expect("load");
        let mut resumed = super::super::JourneySession::from_state(
            restored.rehydrate(data),
            super::super::StrategyId::Balanced,
            &cfg,
        );
        for _ in 0..8 {
            original.tick_day();
            resumed.tick_day();
            assert_eq!(
                serde_json::to_value(original.state()).expect("original"),
                serde_json::to_value(resumed.state()).expect("resumed")
            );
        }
    }
    #[test]
    fn mixed_draws_continue_after_serializing_game_state() {
        let bundle = Rc::new(RngBundle::from_user_seed(42));
        bundle.weather().next_u32();
        bundle.weather().next_u64();
        bundle.health().fill_bytes(&mut [0; 17]);
        let gs = crate::GameState {
            rng_bundle: Some(bundle.clone()),
            ..crate::GameState::default()
        };
        let saved = serde_json::to_string(&gs).expect("serialize");
        let loaded: crate::GameState = serde_json::from_str(&saved).expect("deserialize");
        let restored = loaded.rng_bundle.expect("bundle");
        assert_eq!(restored.weather().next_u64(), bundle.weather().next_u64());
        assert_eq!(restored.health().next_u64(), bundle.health().next_u64());
    }
    #[test]
    fn session_from_state_keeps_stream_position() {
        let bundle = Rc::new(RngBundle::from_user_seed(42));
        bundle.events().next_u64();
        let state = crate::GameState {
            rng_bundle: Some(bundle.clone()),
            ..crate::GameState::default()
        };
        let expected = bundle;
        let session = super::super::JourneySession::from_state(
            state,
            super::super::StrategyId::Balanced,
            &crate::EndgameTravelCfg::default(),
        );
        assert!(Rc::ptr_eq(
            session.state().rng_bundle.as_ref().expect("bundle"),
            &expected
        ));
    }
}
