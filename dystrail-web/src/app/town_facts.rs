//! Local facts are sourced by exact settlement, independently from fictional NPCs.
use crate::{game::GameState, i18n};
use std::collections::BTreeMap;
use yew::prelude::*;
#[derive(serde::Deserialize)]
pub struct TownFact {
    pub town: String,
    #[serde(default)]
    pub title: BTreeMap<String, String>,
    #[serde(default)]
    pub setup: BTreeMap<String, String>,
    #[serde(default)]
    pub sources: Vec<super::town_content::Source>,
    pub text: BTreeMap<String, String>,
    pub source: String,
    pub checked: String,
    pub comment: BTreeMap<String, String>,
}
#[must_use]
pub fn fact(gs: &GameState) -> Option<TownFact> {
    if let Some(c) = super::town_content::selected(gs) {
        return Some(TownFact {
            town: c.town,
            title: c.title,
            setup: c.setup,
            text: c.text,
            comment: c.comment,
            sources: c.sources,
            source: String::new(),
            checked: String::new(),
        });
    }
    let name = super::town::name(gs);
    serde_json::from_str::<Vec<TownFact>>(include_str!("../../static/assets/data/town-facts.json"))
        .ok()?
        .into_iter()
        .find(|f| f.town == name)
}
impl TownFact {
    #[must_use]
    pub fn title(&self) -> String {
        self.title
            .get(&i18n::current_lang())
            .or_else(|| self.title.get("en"))
            .cloned()
            .unwrap_or_else(|| i18n::t("journey.local_word"))
    }
    pub fn setup(&self) -> String {
        self.setup
            .get(&i18n::current_lang())
            .or_else(|| self.setup.get("en"))
            .cloned()
            .unwrap_or_default()
    }
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

/// Reuse sourced endpoint context at departure and arrival without creating a town encounter.
pub fn endpoint_context(town: &str) -> Html {
    let fact = serde_json::from_str::<Vec<TownFact>>(include_str!("../../static/assets/data/town-facts.json"))
        .ok().and_then(|facts| facts.into_iter().find(|fact| fact.town == town));
    let Some(fact) = fact else { return Html::default(); };
    html! {<span class="endpoint-context" data-town={town.to_owned()}>
        <crate::components::ui::context_help::ContextHelp informational={true} icon={"i".to_owned()} title={format!("{} · {}",town,i18n::t("trail.record"))}>
            <p class="endpoint-fact">{fact.message()}</p>
            <p><a href={fact.source.clone()} target="_blank" rel="noopener noreferrer">{i18n::t("trail.source")}</a>{" · "}{i18n::tr("trail.verified",Some(&BTreeMap::from([("date",fact.checked.as_str())])))}</p>
        </crate::components::ui::context_help::ContextHelp>
    </span>}
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
        <crate::components::ui::world_view::WorldView state={std::rc::Rc::new(gs.clone())} title={fact.title()} stage={Some(crate::components::ui::journey_scene::SceneStage::Town)} local_npc={gs.continuity.activities.local_word} />
        <section class="local-conversation" aria-label={i18n::t("journey.local_word")}>
            <div class="conversation-content">
                <div class="resident-story">
                    <div class="resident-byline"><span class="eyebrow">{i18n::t("trail.local")}</span><span>{fact.town.clone()}</span></div>
                    <p>{fact.setup()}</p>
                    <blockquote class="resident-remark"><p>{fact.remark()}</p></blockquote>
                </div>
                <aside class="local-record" aria-labelledby="local-record-heading">
                    <div class="local-record-heading"><h2 id="local-record-heading">{i18n::t("trail.record")}</h2><crate::components::ui::context_help::ContextHelp title={i18n::t("trail.about_conversation")} text={i18n::t("trail.fact_note")} icon={"ⓘ".to_owned()} informational={true} /></div>
                    <p class="local-fact">{fact.message()}</p>
                    if fact.sources.is_empty() {
                        <div class="fact-source"><a href={fact.source.clone()} target="_blank" rel="noopener noreferrer">{i18n::t("trail.source")}</a><span>{i18n::tr("trail.verified",Some(&BTreeMap::from([("date",fact.checked.as_str())])))}</span></div>
                    } else { {for fact.sources.iter().map(|source| html!{<div class="fact-source"><a href={source.url.clone()} target="_blank" rel="noopener noreferrer">{source.title.clone()}</a><span>{i18n::tr("trail.verified",Some(&BTreeMap::from([("date",source.checked.as_str())])))}</span><p>{source.note()}</p></div>})} }
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
