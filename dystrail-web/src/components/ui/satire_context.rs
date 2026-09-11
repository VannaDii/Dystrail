//! The historical hook remains available offline; the linked source is optional reading.
use yew::prelude::*;

pub fn encounter(id: &str) -> Html {
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
    let fact = source["translations"][&language]
        .as_str()
        .or_else(|| source["fact"].as_str())
        .unwrap_or_default();
    let text = format!(
        "{} · {}\n{}",
        source["date"].as_str().unwrap_or_default(),
        fact,
        crate::i18n::t("trail.satire_note")
    );
    html! {<super::context_help::ContextHelp title={crate::i18n::t("trail.behind_joke")} text={text} icon={"ⓘ".to_owned()} source_href={source["source"].as_str().map(str::to_owned)} />}
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
