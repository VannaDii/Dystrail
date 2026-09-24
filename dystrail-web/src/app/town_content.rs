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
        self.notes
            .get(&crate::i18n::current_lang())
            .or_else(|| self.notes.get("en"))
            .cloned()
            .unwrap_or_else(|| self.qualification.clone())
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
    serde_json::from_str(include_str!(
        "../../static/assets/data/town-conversations.json"
    ))
    .expect("validated town conversation catalog")
}

/// A forged or obsolete saved reference cannot move another town's story here.
pub fn selected(gs: &GameState) -> Option<Conversation> {
    let name = super::town::name(gs);
    let catalog = catalog();
    let family = &catalog.iter().find(|c| c.town == name)?.family_id;
    let unit = super::visual_content::service_unit(
        gs,
        family,
        "town",
        gs.continuity.route_services.stop.unwrap_or_default(),
    );
    catalog.into_iter().find(|c| c.town == name && c.id == unit)
}

/// Seal before advancing the action clock, including a repeat conversation.
pub fn seal(gs: &mut GameState) -> Option<String> {
    let story = selected(gs)?;
    let key = format!(
        "{}/town/{}",
        story.family_id,
        gs.continuity.route_services.stop.unwrap_or_default()
    );
    gs.continuity
        .visual_content
        .selections
        .insert(key, story.id.clone());
    Some(story.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state_for(town: &str) -> GameState {
        for route in crate::game::route::routes() {
            if let Some(stop) = route.stops.iter().find(|stop| stop.name == town) {
                let mut gs = GameState {
                    seed: 42,
                    ..GameState::default()
                };
                gs.continuity.route_services.route_id = Some(route.id.clone());
                gs.continuity.route_services.stop = Some(u32::from(stop.mile));
                return gs;
            }
        }
        panic!("missing stop {town}")
    }
    #[test]
    fn all_132_conversations_preserve_town_sources_and_current_save_identity() {
        let rows = catalog();
        assert_eq!(rows.len(), 132);
        let mut families = BTreeMap::<String, Vec<String>>::new();
        for c in rows {
            families
                .entry(c.family_id.clone())
                .or_default()
                .push(c.id.clone());
            for text in [&c.title, &c.text, &c.setup, &c.comment] {
                assert!(!text["en"].is_empty());
            }
            assert!(!c.sources.is_empty());
            assert!(c.sources.iter().all(|s| s.url.starts_with("https://")
                && !s.checked.is_empty()
                && !s.qualification.is_empty()));
            let mut gs = state_for(&c.town);
            let key = format!(
                "{}/town/{}",
                c.family_id,
                gs.continuity.route_services.stop.unwrap()
            );
            gs.continuity
                .visual_content
                .selections
                .insert(key.clone(), c.id.clone());
            assert_eq!(selected(&gs).unwrap().id, c.id);
            let before = serde_json::to_value(&gs).unwrap();
            assert_eq!(seal(&mut gs), Some(c.id.clone()));
            assert_eq!(serde_json::to_value(&gs).unwrap(), before);
            let mut restored: GameState =
                serde_json::from_str(&serde_json::to_string(&gs).unwrap()).unwrap();
            restored.day += 1;
            assert_eq!(selected(&restored).unwrap().id, c.id);
            restored
                .continuity
                .visual_content
                .selections
                .insert(key, "TOWN-99-Z".into());
            assert_eq!(selected(&restored).unwrap().town, c.town);
        }
        assert_eq!(families.len(), 44);
        for (family, ids) in families {
            assert_eq!(ids, ["A", "B", "C"].map(|v| format!("{family}-{v}")));
        }
    }
}
