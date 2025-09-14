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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Persona {
    #[serde(skip)]
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
    pub async fn load() -> Self {
        // Attempt network load first
        let url = "/static/assets/data/personas.json";
        if let Ok(resp) = gloo_net::http::Request::get(url).send().await {
            if resp.status() == 200 {
                if let Ok(map) = resp
                    .json::<std::collections::HashMap<String, PersonaNoId>>()
                    .await
                {
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
                    return PersonasList(v);
                }
            }
        }
        // Fallback to embedded copy (for tests / offline)
        let raw = include_str!("../../static/assets/data/personas.json");
        if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, PersonaNoId>>(raw)
        {
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
            return PersonasList(v);
        }
        PersonasList::default()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn embedded_personas_parse() {
        let raw = include_str!("../../static/assets/data/personas.json");
        let map: std::collections::HashMap<String, PersonaNoId> =
            serde_json::from_str(raw).unwrap();
        assert!(map.contains_key("journalist"));
        let p = Persona::with_id("journalist".into(), map.get("journalist").unwrap().clone());
        assert_eq!(p.name, "Journalist");
        assert!(p.score_mult > 0.0);
    }
}
