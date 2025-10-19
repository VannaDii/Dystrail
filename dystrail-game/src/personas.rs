use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PersonaStart {
    #[serde(default)]
    pub supplies: i32,
    #[serde(default)]
    pub credibility: i32,
    #[serde(default)]
    pub sanity: i32,
    #[serde(default)]
    pub morale: i32,
    #[serde(default)]
    pub allies: i32,
    #[serde(default)]
    pub budget: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PersonaMods {
    #[serde(default)]
    pub receipt_find_pct: i32,
    #[serde(default)]
    pub store_discount_pct: i32,
    #[serde(default)]
    pub eo_heat_pct: i32,
    #[serde(default)]
    pub bribe_discount_pct: i32,
    #[serde(default)]
    pub satire_sustain: bool,
    #[serde(default)]
    pub pants_relief: i32,
    #[serde(default)]
    pub pants_relief_threshold: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub score_mult: f32,
    pub start: PersonaStart,
    #[serde(default)]
    pub mods: PersonaMods,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
struct PersonaNoId {
    pub name: String,
    pub desc: String,
    pub score_mult: f32,
    pub start: PersonaStart,
    #[serde(default)]
    pub mods: PersonaMods,
}

impl Persona {
    #[must_use]
    fn with_id(id: String, p: PersonaNoId) -> Self {
        Self {
            id,
            name: p.name,
            desc: p.desc,
            score_mult: p.score_mult,
            start: p.start,
            mods: p.mods,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PersonasList(pub Vec<Persona>);

impl PersonasList {
    #[must_use]
    pub const fn empty() -> Self {
        Self(vec![])
    }

    /// Load personas from JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON cannot be parsed into valid persona data.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let map: std::collections::HashMap<String, PersonaNoId> = serde_json::from_str(json)?;
        let order = [
            "journalist",
            "organizer",
            "whistleblower",
            "lobbyist",
            "staffer",
            "satirist",
        ];
        let mut v = Vec::with_capacity(order.len());
        for id in order {
            if let Some(p) = map.get(id) {
                v.push(Persona::with_id(id.to_string(), p.clone()));
            }
        }
        Ok(Self(v))
    }

    #[must_use]
    pub fn get_by_id(&self, id: &str) -> Option<&Persona> {
        self.0.iter().find(|p| p.id == id)
    }

    /// Load personas from static assets (function for web compatibility)
    /// This is a placeholder that returns empty data - web implementation should override this
    #[must_use]
    pub const fn load() -> Self {
        Self::empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Persona> {
        self.0.iter()
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> IntoIterator for &'a PersonasList {
    type Item = &'a Persona;
    type IntoIter = std::slice::Iter<'a, Persona>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_json_parsing() {
        let json = r#"{
            "journalist": {
                "name": "Journalist",
                "desc": "A test journalist persona",
                "score_mult": 1.2,
                "start": {
                    "supplies": 8,
                    "credibility": 12
                },
                "mods": {
                    "receipt_find_pct": 15
                }
            }
        }"#;

        let personas = PersonasList::from_json(json).unwrap();
        assert_eq!(personas.len(), 1);

        let journalist = personas.get_by_id("journalist").unwrap();
        assert_eq!(journalist.name, "Journalist");
        assert_eq!(journalist.start.supplies, 8);
        assert_eq!(journalist.mods.receipt_find_pct, 15);
    }
}
