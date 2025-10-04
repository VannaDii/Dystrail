use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ChoiceEffects {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub label: String,
    #[serde(default)]
    pub effects: ChoiceEffects,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EncounterData {
    pub encounters: Vec<Encounter>,
}

impl EncounterData {
    #[must_use]
    pub fn empty() -> Self {
        Self { encounters: vec![] }
    }
}

impl EncounterData {
    pub async fn load_from_static() -> Self {
        let url = "/static/assets/data/game.json";
        if let Ok(resp) = gloo_net::http::Request::get(url).send().await
            && resp.status() == 200
            && let Ok(list) = resp.json::<Vec<Encounter>>().await {
            return Self { encounters: list };
        }
        Self::empty()
    }
}
