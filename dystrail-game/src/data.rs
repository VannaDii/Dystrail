use serde::{Deserialize, Serialize};

/// Effects applied when a choice is selected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Effects {
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
    pub pants: i32,
    #[serde(default)]
    pub add_receipt: Option<String>,
    #[serde(default)]
    pub use_receipt: bool,
    #[serde(default)]
    pub log: Option<String>,
}

/// A choice within an encounter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub label: String,
    #[serde(default)]
    pub effects: Effects,
}

/// An encounter in the game
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

fn default_weight() -> u32 {
    5
}

/// Container for all encounter data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EncounterData {
    pub encounters: Vec<Encounter>,
}

impl EncounterData {
    /// Create empty encounter data (useful for tests)
    #[must_use]
    pub fn empty() -> Self {
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
        serde_json::from_str(json)
    }

    /// Create encounter data from pre-parsed encounters
    #[must_use]
    pub fn from_encounters(encounters: Vec<Encounter>) -> Self {
        Self { encounters }
    }

    /// Load encounter data from static assets (function for web compatibility)
    /// This is a placeholder that returns default data - web implementation should override this
    #[must_use]
    pub fn load_from_static() -> Self {
        // Return default/empty data - web layer should provide actual implementation
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
