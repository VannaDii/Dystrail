//! External contact vignettes never draw from, or change, the traveling party.
use super::SceneStage;
use yew::prelude::*;

pub fn coordinates(unit: &str) -> Option<(usize, char)> {
    let suffix = unit.strip_prefix("ALLY-")?;
    let (family, variant) = suffix.split_once('-')?;
    let family: usize = family.parse().ok()?;
    if !(1..=6).contains(&family) || family.to_string() != family.to_string() {
        return None;
    }
    let variant = match variant { "A" => 'a', "B" => 'b', "C" => 'c', _ => return None };
    if unit != format!("ALLY-{family:02}-{}", variant.to_ascii_uppercase()) { return None; }
    Some((family - 1, variant))
}

pub fn unit(stage: &SceneStage) -> Option<&str> {
    let SceneStage::AllyDeparture { unit } = stage else { return None; };
    coordinates(unit)?;
    Some(unit)
}

pub fn indoors(unit: &str) -> bool {
    !matches!(unit, "ALLY-01-B" | "ALLY-06-A")
}

pub fn render(stage: &SceneStage) -> Option<Html> {
    let unit = unit(stage)?;
    let (cell, variant) = coordinates(unit)?;
    let sign = crate::i18n::t(&format!("visual_copy.{unit}.sign"));
    Some(html! {<div class="ally-composition" data-external-contact="true" data-ally-unit={unit.to_owned()}>
        <svg class="scene-background" viewBox={format!("{} {} 768 512", cell % 2 * 768, cell / 2 * 512)} preserveAspectRatio="xMidYMid slice" overflow="hidden">
            <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/ally-settings-{variant}-20260915.png"))} width="1536" height="1536" preserveAspectRatio="none" />
        </svg>
        if !sign.is_empty() {<div class="ally-sign" dir="auto">{sign}</div>}
    </div>})
}
