//! The historical hook remains available offline; the linked source is optional reading.
use yew::prelude::*;

pub fn encounter(id: &str) -> Html {
    let references = serde_json::from_str::<std::collections::BTreeMap<String, Vec<String>>>(include_str!(
        "../../../static/assets/data/road-source-references.json"
    )).expect("validated road source references");
    if let Some(references) = references.get(id) {
        let fact = if super::journey_scene::road_art::supported(id) {
            crate::i18n::encounter_text(id, "fact", "")
        } else { String::new() };
        return html! {<super::context_help::ContextHelp informational={true} title={crate::i18n::t("trail.behind_joke")}>
            <div class="source-explanation" data-source-unit={id.to_owned()}>
                {if fact.is_empty() {Html::default()} else {html!{<p>{fact}</p>}}}
                <ul lang="en" dir="auto">{for references.iter().map(|reference| html!{<li class="source-reference">{reference}</li>})}</ul>
                <p class="source-note">{crate::i18n::t("trail.satire_note")}</p>
            </div>
        </super::context_help::ContextHelp>};
    }
    if super::journey_scene::road_art::supported(id) {
        let fact = crate::i18n::encounter_text(id, "fact", "");
        return html! {<super::context_help::ContextHelp informational={true} title={crate::i18n::t("trail.behind_joke")} text={fact} />};
    }
    let bank = serde_json::from_str::<Vec<serde_json::Value>>(include_str!(
        "../../../static/assets/data/game.json"
    ))
    .unwrap_or_default();
    bank.iter()
        .find(|e| e["id"] == id)
        .and_then(|e| e["satire_hook"].as_str())
        .map_or_else(Html::default, hook)
}
pub fn hook(id: &str) -> Html {
    let sources = serde_json::from_str::<serde_json::Value>(include_str!(
        "../../../static/assets/data/satire-sources.json"
    ))
    .unwrap_or_default();
    let Some(source) = sources.get(id) else {
        return Html::default();
    };
    let language = crate::i18n::current_lang();
    let translation = source["translations"][&language].as_str();
    let fact = translation
        .or_else(|| source["fact"].as_str())
        .unwrap_or_default();
    let date = source["date"].as_str().unwrap_or_default();
    html! {<super::context_help::ContextHelp informational={true} title={crate::i18n::t("trail.behind_joke")} icon={"ⓘ".to_owned()} source_href={source["source"].as_str().map(str::to_owned)}>
        <div class="source-explanation">
            <time datetime={date.to_owned()}>{crate::i18n::fmt_date_iso(date)}</time>
            <p lang={if translation.is_some() {language.as_str()} else {"en"}.to_owned()} dir="auto">{fact}</p>
            <p class="source-note">{crate::i18n::t("trail.satire_note")}</p>
        </div>
    </super::context_help::ContextHelp>}
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_encounter_has_a_source_and_an_affordable_way_through() {
        let sources: serde_json::Value = serde_json::from_str(include_str!(
            "../../../static/assets/data/satire-sources.json"
        ))
        .unwrap();
        let bank: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../../../static/assets/data/game.json")).unwrap();
        for event in bank {
            let id = event["satire_hook"].as_str().unwrap();
            assert!(
                sources[id]["source"]
                    .as_str()
                    .unwrap()
                    .starts_with("https://")
            );
            let event: crate::game::data::Encounter = serde_json::from_value(event).unwrap();
            let stats = crate::game::Stats {
                supplies: 0,
                ..crate::game::Stats::default()
            };
            assert!(
                event
                    .choices
                    .iter()
                    .any(|c| c.effects.affordable(&stats, 0, 0)),
                "{}",
                event.id
            );
        }
    }
}
