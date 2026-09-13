use serde::{Deserialize, Serialize};

/// Effects applied when a choice is selected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Effects {
    #[serde(default)]
    pub cash_cents: i64,
    #[serde(default)]
    pub hp: i32,
    #[serde(default)]
    pub sanity: i32,
    #[serde(default)]
    pub credibility: i32,
    #[serde(default)]
    pub supplies: i32,
    #[serde(default)]
    pub morale: i32,
    #[serde(default)]
    pub allies: i32,
    #[serde(default)]
    pub travel_bonus_ratio: f32,
    #[serde(default)]
    pub add_receipt: Option<String>,
    #[serde(default)]
    pub use_receipt: bool,
    #[serde(default)]
    pub log: Option<String>,
    #[serde(default)]
    pub rest: bool,
}

impl Effects {
    /// Mandatory costs must be payable before an encounter can be resolved.
    #[must_use]
    pub const fn affordable(&self, stats: &crate::Stats, cash: i64, receipts: usize) -> bool {
        cash.saturating_add(self.cash_cents) >= 0
            && stats.supplies.saturating_add(self.supplies) >= 0
            && (!self.use_receipt || receipts > 0)
    }
}

/// A choice within an encounter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub label: String,
    #[serde(default)]
    pub effects: Effects,
}

/// An encounter in the game
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Encounter {
    pub id: String,
    pub name: String,
    pub desc: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub regions: Vec<String>,
    #[serde(default)]
    pub modes: Vec<String>,
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub hard_stop: bool,
    #[serde(default)]
    pub major_repair: bool,
    #[serde(default)]
    pub chainable: bool,
}

const fn default_weight() -> u32 {
    5
}

/// Container for all encounter data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EncounterData {
    pub encounters: Vec<Encounter>,
}

impl EncounterData {
    /// Create empty encounter data (useful for tests)
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            encounters: Vec::new(),
        }
    }

    /// Load encounter data from JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON cannot be parsed into valid encounter data.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json).or_else(|_| {
            let encounters: Vec<Encounter> = serde_json::from_str(json)?;
            Ok(Self { encounters })
        })
    }

    /// Create encounter data from pre-parsed encounters
    #[must_use]
    pub const fn from_encounters(encounters: Vec<Encounter>) -> Self {
        Self { encounters }
    }

    /// Load encounter data from static assets (function for web compatibility)
    /// This is a placeholder that returns default data - web implementation should override this
    #[must_use]
    pub const fn load_from_static() -> Self {
        // Return default/empty data - web layer should provide actual implementation
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_region_offers_evidence_in_both_modes() {
        let data = EncounterData::from_json(include_str!(
            "../../dystrail-web/static/assets/data/game.json"
        ))
        .unwrap();
        for region in [
            "PacificCoast",
            "MountainWest",
            "Southwest",
            "Heartland",
            "RustBelt",
            "Beltway",
        ] {
            for mode in ["classic", "deep_end"] {
                assert!(
                    data.encounters.iter().any(|event| {
                        event.regions.iter().any(|r| r == region)
                            && event.modes.iter().any(|m| m == mode)
                            && event
                                .choices
                                .iter()
                                .any(|choice| choice.effects.add_receipt.is_some())
                    }),
                    "No evidence opportunity in {region}/{mode}"
                );
            }
        }
    }

    #[test]
    fn shipped_evidence_choices_award_save_and_score_their_receipt_once() {
        let data = EncounterData::from_json(include_str!(
            "../../dystrail-web/static/assets/data/game.json"
        ))
        .unwrap();
        for event in data.encounters {
            for (index, choice) in event.choices.iter().enumerate() {
                let Some(receipt) = &choice.effects.add_receipt else {
                    continue;
                };
                let mut state = crate::GameState {
                    budget_cents: 10_000,
                    current_encounter: Some(event.clone()),
                    ..crate::GameState::default()
                };
                state.apply_choice(index);
                assert_eq!(state.receipts, vec![receipt.clone()], "{}", event.id);
                let saved = serde_json::to_string(&state).unwrap();
                let mut restored: crate::GameState = serde_json::from_str(&saved).unwrap();
                restored.apply_choice(index);
                assert_eq!(restored.receipts, state.receipts);
                let score = restored.journey_score();
                restored.receipts.clear();
                assert_eq!(score - restored.journey_score(), 8);
            }
        }
    }

    #[test]
    fn mandatory_cash_and_supply_costs_are_atomic_and_rewards_reach_the_wallet() {
        let data = EncounterData::from_json(r#"[{"id":"cash","name":"Shift","desc":"Work","choices":[{"label":"Pay","effects":{"cash_cents":-500,"supplies":-2}},{"label":"Work","effects":{"cash_cents":1200,"sanity":-1}}]}]"#).unwrap();
        let mut gs = crate::GameState {
            budget_cents: 400,
            current_encounter: Some(data.encounters[0].clone()),
            ..crate::GameState::default()
        };
        let before = gs.stats.clone();
        gs.apply_choice(0);
        assert!(gs.current_encounter.is_some());
        assert_eq!(gs.budget_cents, 400);
        assert_eq!(gs.stats, before);
        gs.apply_choice(1);
        assert_eq!(gs.budget_cents, 1600);
        assert_eq!(gs.budget, 16);
        assert!(gs.current_encounter.is_none());
        gs.apply_choice(1);
        assert_eq!(gs.budget_cents, 1600);
    }

    #[test]
    fn test_encounter_data_from_json() {
        let json = r#"{
            "encounters": [
                {
                    "id": "test1",
                    "name": "Test Encounter",
                    "desc": "A test encounter",
                    "choices": [
                        {
                            "label": "Do something",
                            "effects": {
                                "hp": -1,
                                "supplies": 2
                            }
                        }
                    ]
                }
            ]
        }"#;

        let data = EncounterData::from_json(json).unwrap();
        assert_eq!(data.encounters.len(), 1);
        assert_eq!(data.encounters[0].name, "Test Encounter");
        assert_eq!(data.encounters[0].choices[0].effects.hp, -1);
        assert_eq!(data.encounters[0].choices[0].effects.supplies, 2);
    }

    #[test]
    fn test_encounter_data_from_array() {
        let json = r#"[
            {
                "id": "array1",
                "name": "Array Encounter",
                "desc": "Encounter represented directly as array element",
                "choices": [
                    {
                        "label": "Proceed",
                        "effects": {
                            "hp": 1,
                            "supplies": -1
                        }
                    }
                ]
            }
        ]"#;

        let data = EncounterData::from_json(json).unwrap();
        assert_eq!(data.encounters.len(), 1);
        assert_eq!(data.encounters[0].id, "array1");
        assert_eq!(data.encounters[0].choices[0].effects.hp, 1);
    }
}
