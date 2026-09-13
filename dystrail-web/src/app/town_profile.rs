//! Geography and visitor information are separate from the resident's political story.
use crate::{game::GameState, i18n};
use std::collections::BTreeMap;
use yew::prelude::*;

#[derive(serde::Deserialize)]
struct Profile {
    town: String,
    state: String,
    population: u32,
    population_year: u16,
    population_source: String,
    attraction: BTreeMap<String, String>,
    attraction_source: String,
}

pub fn render(gs: &GameState) -> Html {
    let profile = serde_json::from_str::<Vec<Profile>>(include_str!(
        "../../static/assets/data/town-profiles.json"
    ))
    .ok()
    .and_then(|profiles| {
        profiles
            .into_iter()
            .find(|p| p.town == super::town::name(gs))
    });
    let Some(p) = profile else {
        return Html::default();
    };
    let attraction = p
        .attraction
        .get(&i18n::current_lang())
        .or_else(|| p.attraction.get("en"));
    html! {<div class="town-profile">
        <dl class="town-vitals"><div><dt>{i18n::t("town_profile.state")}</dt><dd>{p.state}</dd></div><div><dt>{i18n::t("town_profile.population").replace("{year}",&p.population_year.to_string())}</dt><dd><a href={p.population_source} target="_blank" rel="noopener noreferrer">{i18n::fmt_number(f64::from(p.population))}</a></dd></div></dl>
        if let Some(attraction) = attraction {<div class="town-attraction"><h3>{i18n::t("town_profile.attractions")}</h3><p>{attraction}</p><a href={p.attraction_source} target="_blank" rel="noopener noreferrer">{i18n::t("trail.source")}</a></div>}
    </div>}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_route_stop_has_a_unique_sourced_profile() {
        let profiles: Vec<Profile> =
            serde_json::from_str(include_str!("../../static/assets/data/town-profiles.json"))
                .unwrap();
        let names: std::collections::BTreeSet<_> =
            profiles.iter().map(|p| p.town.as_str()).collect();
        assert_eq!(names.len(), profiles.len());
        for route in crate::game::route::routes() {
            for stop in &route.stops {
                let profile = profiles
                    .iter()
                    .find(|p| p.town == stop.name)
                    .unwrap_or_else(|| panic!("Missing profile for {}", stop.name));
                assert!(profile.population > 0 && !profile.state.is_empty());
                assert!(profile.population_year >= 2025);
                assert!(
                    profile
                        .population_source
                        .ends_with(&format!("/PST0452{}", profile.population_year % 100))
                );
                assert!(
                    profile
                        .population_source
                        .starts_with("https://www.census.gov/")
                );
                assert!(profile.attraction_source.starts_with("https://"));
                for language in ["en", "es", "it", "ar"] {
                    assert!(
                        profile
                            .attraction
                            .get(language)
                            .is_some_and(|text| !text.is_empty())
                    );
                }
            }
        }
    }
}
