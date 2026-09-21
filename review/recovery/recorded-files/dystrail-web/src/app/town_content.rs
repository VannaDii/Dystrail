//! Each fictional conversation travels with its own factual record and sources.
//! Presentation selection never consumes the journey's random stream.
use crate::game::GameState;
use std::collections::BTreeMap;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Source {
    pub url: String,
    pub title: String,
    pub checked: String,
    pub qualification: String,
    #[serde(default)]
    pub notes: BTreeMap<String, String>,
}

impl Source {
    pub fn note(&self) -> String {
        self.notes.get(&crate::i18n::current_lang()).or_else(|| self.notes.get("en"))
            .cloned().unwrap_or_else(|| self.qualification.clone())
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Conversation {
    pub id: String,
    pub family_id: String,
    pub town: String,
    pub title: BTreeMap<String, String>,
    pub text: BTreeMap<String, String>,
    pub setup: BTreeMap<String, String>,
    pub comment: BTreeMap<String, String>,
    pub sources: Vec<Source>,
}

fn catalog() -> Vec<Conversation> {
    serde_json::from_str(include_str!("../../static/assets/data/town-conversations.json"))
        .expect("validated town conversation catalog")
}

fn occurrence(gs: &GameState) -> String {
    format!(
        "town/{}/{}",
        gs.continuity.route_services.route_id.as_deref().unwrap_or_default(),
        gs.continuity.route_services.stop.unwrap_or_default(),
    )
}

/// A forged or obsolete saved reference cannot move another town's story here.
pub fn selected(gs: &GameState) -> Option<Conversation> {
    let name = super::town::name(gs);
    let catalog = catalog();
    let family = &catalog.iter().find(|c| c.town == name)?.family_id;
    let unit = super::visual_content::selected(gs, family, &occurrence(gs));
    catalog.into_iter().find(|c| c.town == name && c.id == unit)
}

/// Seal before advancing the action clock, including a repeat conversation.
pub fn seal(gs: &mut GameState) -> Option<String> {
    let story = selected(gs)?;
    Some(super::visual_content::select(gs, &story.family_id, &occurrence(gs)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn town_state(name: &str) -> GameState {
        let routes: serde_json::Value =
            serde_json::from_str(include_str!("../../../dystrail-game/data/routes.json")).unwrap();
        for route in routes.as_array().unwrap() {
            if let Some(stop) = route["stops"].as_array().unwrap().iter().find(|s| s["name"] == name) {
                let mut gs = GameState::default();
                gs.continuity.route_services.route_id = Some(route["id"].as_str().unwrap().into());
                gs.continuity.route_services.stop = Some(stop["mile"].as_u64().unwrap() as u32);
                return gs;
            }
        }
        panic!("missing town {name}")
    }

    #[test]
    fn every_town_family_has_three_complete_source_bound_variants() {
        let catalog = catalog();
        assert_eq!(catalog.len(), 132);
        let mut families = BTreeMap::<String, Vec<String>>::new();
        for c in &catalog {
            families.entry(c.family_id.clone()).or_default().push(c.id.clone());
            for copy in [&c.title, &c.text, &c.setup, &c.comment] {
                assert!(!copy["en"].is_empty(), "{}", c.id);
            }
            assert!(!c.sources.is_empty());
            assert!(c.sources.iter().all(|s| s.url.starts_with("https://") && !s.checked.is_empty()));
            let mut gs = town_state(&c.town);
            gs.continuity.visual_content.selections.insert(
                format!("{}/{}", c.family_id, occurrence(&gs)), c.id.clone());
            assert_eq!(selected(&gs).unwrap().id, c.id);
        }
        assert_eq!(families.len(), 44);
        for (family, ids) in families {
            assert_eq!(ids, ["A", "B", "C"].map(|v| format!("{family}-{v}")));
        }
    }

    #[test]
    fn town_selection_survives_reload_and_clock_changes_without_engine_changes() {
        let mut gs = town_state("Albuquerque");
        let original = serde_json::to_value(&gs).unwrap();
        let first = seal(&mut gs).unwrap();
        let mut after = serde_json::to_value(&gs).unwrap();
        after["visual_content"] = original["visual_content"].clone();
        assert_eq!(after, original);
        let mut restored: GameState = serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
        restored.day += 1;
        assert_eq!(seal(&mut restored).unwrap(), first);
        restored.continuity.visual_content.selections.insert(
            format!("TOWN-01/{}", occurrence(&restored)), "TOWN-02-B".into());
        assert_eq!(selected(&restored).unwrap().id, "TOWN-01-A");
    }
}
