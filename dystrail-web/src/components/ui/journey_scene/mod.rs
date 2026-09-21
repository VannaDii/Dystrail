//! Stage-specific presentation; artwork never participates in the simulation.
use crate::game::Region;
use yew::prelude::*;
pub mod billboard;
pub mod composition;
mod lighting;
pub(crate) mod parked;
pub mod road_art;
mod van;

/// Explicit scene context. An aftermath keeps the context of its triggering action.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SceneStage {
    Setup,
    Travel(Region),
    Camp,
    Town,
    Care,
    Ending(bool),
    Breakdown,
    Encounter(String),
    EncounterOutcome { unit: String, choice: usize },
    Boss,
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub local_npc: Option<u8>,
    #[prop_or_default]
    pub subject: Option<String>,
    #[prop_or_default]
    pub region: Option<Region>,
    pub stage: SceneStage,
    #[prop_or_default]
    pub deep: bool,
    #[prop_or_default]
    pub weather: Option<crate::game::weather::Weather>,
    #[prop_or_default]
    pub moving: bool,
    #[prop_or(1)]
    pub day: u32,
    #[prop_or_default]
    pub seed: u64,
    #[prop_or_default]
    pub road_asset: Option<String>,
    #[prop_or(12)]
    pub hour: u8,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub party: Option<crate::game::party::Party>,
    #[prop_or(true)]
    pub show_cast: bool,
}

/// Only display illustrations that actually depict the current stage or encounter.
#[must_use]
pub fn asset_name(deep: bool, stage: &SceneStage) -> Option<&'static str> {
    match stage {
        SceneStage::Encounter(id) => encounters::asset(id, deep),
        SceneStage::EncounterOutcome { unit, .. } => encounters::asset(unit, deep),
        SceneStage::Breakdown => Some("enc-service"),
        SceneStage::Camp | SceneStage::Care => Some("recovery-camp"),
        SceneStage::Town => Some("town-arrival"),
        SceneStage::Ending(true) => Some("ending-dc"),
        SceneStage::Ending(false) => Some("ending-rest-area"),
        SceneStage::Boss => Some("enc-civic"),
        SceneStage::Travel(region) => Some(road_asset(*region, 1)),
        SceneStage::Setup => Some("open-heartland-orchard"),
    }
}
mod encounters;

#[must_use]
pub const fn road_asset(region: Region, day: u32) -> &'static str {
    match region {
        Region::PacificCoast => "open-pacific-northwest",
        Region::MountainWest => "open-mountain-west",
        Region::Southwest => "open-southwest-desert",
        Region::Heartland => "open-heartland-orchard",
        Region::RustBelt => "open-rustbelt-lakeside",
        Region::Beltway if day % 2 == 0 => "open-beltway-suburbs",
        Region::Beltway => "open-beltway-parkway",
    }
}

/// Saved route overrides must not restore excluded artwork. Keep only reviewed roads.
fn reviewed_road(region: Region, day: u32, requested: Option<&str>) -> &'static str {
    match requested {
        Some("open-pacific-northwest") => "open-pacific-northwest",
        Some("open-california-hills") => "open-california-hills",
        Some("open-mountain-west") => "open-mountain-west",
        Some("open-southwest-desert") => "open-southwest-desert",
        Some("open-heartland-orchard") => "open-heartland-orchard",
        Some("open-rustbelt-lakeside") => "open-rustbelt-lakeside",
        Some("open-beltway-parkway") => "open-beltway-parkway",
        Some("open-beltway-suburbs") => "open-beltway-suburbs",
        Some("open-great-basin") => "open-great-basin",
        _ => road_asset(region, day),
    }
}

#[function_component(JourneyScene)]
pub fn journey_scene(p: &Props) -> Html {
    crate::i18n::use_language();
    let road = matches!(p.stage, SceneStage::Travel(_) | SceneStage::Setup);
    let stopped = !p.moving && matches!(p.stage, SceneStage::Travel(_));
    let name = if let SceneStage::Travel(region) = p.stage {
        Some(reviewed_road(region, p.day, p.road_asset.as_deref()))
    } else {
        asset_name(p.deep, &p.stage)
    };
    let weather = p.weather.map_or("clear", |w| match w {
        crate::game::weather::Weather::Clear => "clear",
        crate::game::weather::Weather::Storm => "storm",
        crate::game::weather::Weather::ColdSnap => "cold",
        crate::game::weather::Weather::HeatWave => "heat",
        crate::game::weather::Weather::Smoke => "smoke",
    });
    let light = lighting::profile(p.hour);
    let indoors = road_art::context(&p.stage).map_or_else(
        || name.is_some_and(composition::is_indoors),
        |(unit, _)| unit == "ENC-C01-B",
    );
    let authored = road_art::aspect(&p.stage);
    let authored_style =
        authored.map_or(String::new(), |ratio| format!("--authored-ratio:{ratio}"));
    html! { <figure style={authored_style} data-weather={weather} data-time={light} data-hour={p.hour.to_string()} data-indoors={indoors.to_string()} data-scene={name.unwrap_or("unillustrated").to_owned()} class={classes!("journey-scene",authored.is_some().then_some("scene-authored"),road.then_some("scene-road"),p.moving.then_some("scene-moving"))}>
        <div class="scene-art" aria-hidden="true">
            if let Some(art)=road_art::render(&p.stage) {{art}} else if let Some(art)=name.and_then(|name|composition::setting(p,name)) {{art}} else if road {<div class="road-pan-track">{for (0..2).map(|i|html!{<img class={classes!("scene-background",(i==1).then_some("road-mirrored"))} src={crate::paths::asset_path(&format!("static/img/journey/{}.png",name.unwrap_or("open-heartland-orchard")))} alt="" decoding="sync" width="1536" height="1024" />})}</div>} else if let Some(name)=name {<img class="scene-background" src={crate::paths::asset_path(&format!("static/img/journey/{name}.png"))} alt="" decoding="sync" width="1536" height="1024" />}
            if let SceneStage::Travel(region) = p.stage {{billboard::render(billboard::selected(region, p.seed, p.day), p.day)}}
            if stopped {{parked::render(p.party.as_ref())}} else if road {<van::CrewVan party={p.party.clone()} />}
            if road {<div class="road-foreground" aria-hidden="true"><div class="road-pan-track">{for (0..2).map(|i|html!{<img class={classes!("scene-background",(i==1).then_some("road-mirrored"))} src={crate::paths::asset_path(&format!("static/img/journey/{}.png",name.unwrap_or("open-heartland-orchard")))} alt="" decoding="sync" width="1536" height="1024" />})}</div></div>}
            <div class="scene-light"></div>
            <div class="weather-atmosphere"></div>
        </div>
        {for p.children.iter()}
    </figure> }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excluded_and_unknown_saved_overrides_fall_back_to_reviewed_roads() {
        for day in [0, 1, 2, 99] {
            for (region, excluded, expected) in [
                (
                    Region::Heartland,
                    "open-heartland-prairie",
                    "open-heartland-orchard",
                ),
                (
                    Region::RustBelt,
                    "open-rustbelt-foundry",
                    "open-rustbelt-lakeside",
                ),
            ] {
                assert_eq!(reviewed_road(region, day, Some(excluded)), expected);
                assert_eq!(reviewed_road(region, day, Some("unknown")), expected);
            }
        }
        assert_eq!(
            reviewed_road(Region::PacificCoast, 1, Some("open-california-hills")),
            "open-california-hills"
        );
        assert_eq!(
            asset_name(false, &SceneStage::Setup),
            Some("open-heartland-orchard")
        );
    }

    #[test]
    fn roads_use_reviewed_regional_art_without_rng() {
        for region in [Region::Heartland, Region::RustBelt] {
            assert_eq!(road_asset(region, 1), road_asset(region, 2));
            assert_eq!(road_asset(region, 1), road_asset(region, 3));
        }
        assert_ne!(
            road_asset(Region::Beltway, 1),
            road_asset(Region::Beltway, 2)
        );
        assert_eq!(
            reviewed_road(Region::MountainWest, 1, Some("open-great-basin")),
            "open-great-basin"
        );
        assert_eq!(
            reviewed_road(Region::Beltway, 1, Some("open-beltway-suburbs")),
            "open-beltway-suburbs"
        );
        assert_ne!(
            road_asset(Region::Heartland, 1),
            road_asset(Region::RustBelt, 1)
        );
        assert_ne!(
            road_asset(Region::RustBelt, 1),
            road_asset(Region::Beltway, 1)
        );
        assert_eq!(
            asset_name(false, &SceneStage::Encounter("unknown".into())),
            None
        );
    }
}
