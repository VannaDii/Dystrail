//! Local facts are sourced by exact settlement, independently from fictional NPCs.
use crate::{game::GameState, i18n};
use std::collections::BTreeMap;
use yew::prelude::*;
#[derive(serde::Deserialize)]
pub struct TownFact {
    pub town: String,
    pub text: BTreeMap<String, String>,
    pub source: String,
    pub checked: String,
    pub comment: BTreeMap<String, String>,
}
#[must_use]
pub fn fact(gs: &GameState) -> Option<TownFact> {
    let name = super::town::name(gs);
    serde_json::from_str::<Vec<TownFact>>(include_str!("../../static/assets/data/town-facts.json"))
        .ok()?
        .into_iter()
        .find(|f| f.town == name)
}
impl TownFact {
    #[must_use]
    pub fn message(&self) -> String {
        self.text
            .get(&i18n::current_lang())
            .or_else(|| self.text.get("en"))
            .cloned()
            .unwrap_or_default()
    }
    #[must_use]
    pub fn remark(&self) -> String {
        self.comment
            .get(&i18n::current_lang())
            .or_else(|| self.comment.get("en"))
            .cloned()
            .unwrap_or_default()
    }
}
pub fn render(app: &super::state::AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(crate::game::JourneySession::state) else {
        return Html::default();
    };
    let Some(fact) = fact(gs) else {
        return Html::default();
    };
    let done = {
        let app = app.clone();
        Callback::from(move |_| {
            let Some(mut session) = (*app.session).clone() else {
                return;
            };
            session.with_state_mut(|gs| gs.continuity.activities.local_word = None);
            app.session.set(Some(session));
        })
    };
    html! {<>
        <crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={i18n::t("journey.local_word")} stage={Some(crate::components::ui::journey_scene::SceneStage::Town)} local_npc={gs.continuity.activities.local_word} />
        <section class="local-conversation" aria-label={i18n::t("journey.local_word")}>
            <div class="conversation-content">
                <div class="resident-story">
                    <div class="resident-byline"><span class="eyebrow">{i18n::t("trail.local")}</span><span>{fact.town.clone()}</span></div>
                    <blockquote class="resident-remark"><p>{fact.remark()}</p></blockquote>
                </div>
                <aside class="local-record" aria-labelledby="local-record-heading">
                    <div class="local-record-heading"><h2 id="local-record-heading">{i18n::t("trail.record")}</h2><crate::components::ui::context_help::ContextHelp title={i18n::t("trail.about_conversation")} text={i18n::t("trail.fact_note")} icon={"ⓘ".to_owned()} informational={true} /></div>
                    <p class="local-fact">{fact.message()}</p>
                    <div class="fact-source"><a href={fact.source} target="_blank" rel="noopener noreferrer">{i18n::t("trail.source")}</a><span>{i18n::tr("trail.verified",Some(&BTreeMap::from([("date",fact.checked.as_str())])))}</span></div>
                </aside>
            </div>
            <div class="conversation-actions"><button class="retro-btn-primary" onclick={done}>{i18n::t("trail.listen")}</button></div>
        </section>
    </>}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_real_route_stop_has_a_localized_sourced_fact() {
        let facts: Vec<TownFact> =
            serde_json::from_str(include_str!("../../static/assets/data/town-facts.json")).unwrap();
        let routes: serde_json::Value =
            serde_json::from_str(include_str!("../../../dystrail-game/data/routes.json")).unwrap();
        for route in routes.as_array().unwrap() {
            for stop in route["stops"].as_array().unwrap() {
                let f = facts
                    .iter()
                    .find(|f| f.town == stop["name"].as_str().unwrap())
                    .unwrap();
                assert!(f.source.starts_with("https://"));
                for lang in ["en", "it", "es", "ar"] {
                    assert!(!f.text[lang].is_empty());
                }
            }
        }
    }
}
