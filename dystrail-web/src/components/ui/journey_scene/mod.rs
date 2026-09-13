//! Stage-specific presentation; artwork never participates in the simulation.
use crate::game::Region;
use yew::prelude::*;
pub mod composition;
mod parked;
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
        SceneStage::Breakdown => Some("enc-service"),
        SceneStage::Camp | SceneStage::Care => Some("recovery-camp"),
        SceneStage::Town => Some("town-arrival"),
        SceneStage::Ending(true) => Some("ending-dc"),
        SceneStage::Ending(false) => Some("ending-rest-area"),
        SceneStage::Boss => Some("enc-civic"),
        SceneStage::Travel(region) => Some(road_asset(*region, 1)),
        SceneStage::Setup => Some("open-heartland-prairie"),
    }
}
mod encounters;

#[must_use]
pub const fn road_asset(region: Region, day: u32) -> &'static str {
    match (region, day % 2) {
        (Region::PacificCoast, _) => "open-pacific-northwest",
        (Region::MountainWest, _) => "open-mountain-west",
        (Region::Southwest, _) => "open-southwest-desert",
        (Region::Heartland, 0) => "open-heartland-orchard",
        (Region::Heartland, _) => "open-heartland-prairie",
        (Region::RustBelt, 0) => "open-rustbelt-lakeside",
        (Region::RustBelt, _) => "open-rustbelt-foundry",
        (Region::Beltway, 0) => "open-beltway-suburbs",
        (Region::Beltway, _) => "open-beltway-parkway",
    }
}

#[function_component(JourneyScene)]
pub fn journey_scene(p: &Props) -> Html {
    crate::i18n::use_language();
    let road = matches!(p.stage, SceneStage::Travel(_) | SceneStage::Setup);
    let stopped = !p.moving && matches!(p.stage, SceneStage::Travel(_));
    let name = if let SceneStage::Travel(region) = p.stage {
        Some(
            p.road_asset
                .as_deref()
                .unwrap_or_else(|| road_asset(region, p.day)),
        )
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
    let light = match p.hour {
        0..=5 | 21..=23 => "night",
        6..=9 => "morning",
        17..=20 => "dusk",
        _ => "day",
    };
    let indoors = name.is_some_and(composition::is_indoors);
    html! { <figure data-weather={weather} data-time={light} data-hour={p.hour.to_string()} data-indoors={indoors.to_string()} data-scene={name.unwrap_or("unillustrated").to_owned()} class={classes!("journey-scene",road.then_some("scene-road"),p.moving.then_some("scene-moving"))}>
        <div class="scene-art" aria-hidden="true">
            if let Some(art)=name.and_then(|name|composition::setting(p,name)) {{art}} else if road {<div class="road-pan-track">{for (0..2).map(|i|html!{<img class={classes!("scene-background",(i==1).then_some("road-mirrored"))} src={crate::paths::asset_path(&format!("static/img/journey/{}.png",name.unwrap_or("open-heartland-prairie")))} alt="" decoding="sync" width="1536" height="1024" />})}</div>} else if let Some(name)=name {<img class="scene-background" src={crate::paths::asset_path(&format!("static/img/journey/{name}.png"))} alt="" decoding="sync" width="1536" height="1024" />}
            if stopped {{parked::render(p.party.as_ref())}} else if road {<van::CrewVan party={p.party.clone()} />}
            if road {<div class="road-foreground" aria-hidden="true"><div class="road-pan-track">{for (0..2).map(|i|html!{<img class={classes!("scene-background",(i==1).then_some("road-mirrored"))} src={crate::paths::asset_path(&format!("static/img/journey/{}.png",name.unwrap_or("open-heartland-prairie")))} alt="" decoding="sync" width="1536" height="1024" />})}</div></div>}
            if !road && p.show_cast {{composition::cast(p,name.unwrap_or("scene"))}}
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
    fn roads_change_by_region_and_day_without_rng() {
        for region in [Region::Heartland, Region::RustBelt, Region::Beltway] {
            assert_ne!(road_asset(region, 1), road_asset(region, 2));
            assert_eq!(road_asset(region, 1), road_asset(region, 3));
        }
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
