//! Evidence collected through explicit encounter choices.
use crate::GameState;
use rand::Rng;

impl GameState {
    /// Chance of finding one supporting record in addition to a choice's receipt.
    #[must_use]
    pub fn receipt_bonus_chance(&self) -> u8 {
        u8::try_from((10 + self.mods.receipt_find_pct + self.receipt_bonus_pct).clamp(0, 100))
            .unwrap_or(0)
    }

    pub(crate) fn collect_receipt(&mut self, record: &str) {
        self.receipts.push(record.to_owned());
        let chance = self.receipt_bonus_chance();
        let bonus = self
            .rng_bundle
            .as_ref()
            .is_some_and(|rng| rng.events().gen_range(0..100_u8) < chance);
        if bonus {
            self.receipts.push(format!("{record}:supporting-record"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_evidence_is_guaranteed_and_bonus_is_bounded() {
        for (modifier, expected) in [(-100, 1), (100, 2)] {
            let mut state = GameState::default().with_seed(
                42,
                crate::GameMode::Classic,
                crate::EncounterData::empty(),
            );
            state.mods.receipt_find_pct = modifier;
            state.collect_receipt("public-record");
            assert_eq!(state.receipts.len(), expected);
        }
    }

    #[test]
    fn news_diet_bonus_does_not_accumulate_across_days() {
        let mut state = GameState::default();
        state.mods.receipt_find_pct = 15;
        state.diet = crate::state::DietId::Doom;
        let pacing = crate::PacingConfig::default_config();
        state.apply_pace_and_diet(&pacing);
        let first = state.receipt_bonus_chance();
        state.apply_pace_and_diet(&pacing);
        assert_eq!(state.receipt_bonus_chance(), first);
        state.diet = crate::state::DietId::Quiet;
        state.apply_pace_and_diet(&pacing);
        assert!(state.receipt_bonus_chance() < first);
    }
}
