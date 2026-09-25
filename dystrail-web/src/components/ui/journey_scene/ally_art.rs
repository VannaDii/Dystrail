//! Reviewed outside-contact cells, never members of the traveling party.
use super::SceneStage;
use yew::prelude::*;

pub fn coordinates(unit: &str) -> Option<(u32, char)> {
    match unit {
        "ALLY-02-A" => Some((1, 'a')),
        "ALLY-02-C" => Some((1, 'c')),
        "ALLY-04-B" => Some((3, 'b')),
        "ALLY-05-A" => Some((4, 'a')),
        _ => None,
    }
}

pub fn context(stage: &SceneStage) -> Option<(&str, u32, char)> {
    let SceneStage::Encounter(unit) = stage else { return None; };
    let (cell, variant) = coordinates(unit)?;
    Some((unit, cell, variant))
}

pub fn render(stage: &SceneStage) -> Option<Html> {
    let (unit, cell, variant) = context(stage)?;
    let x = cell % 2 * 627;
    let y = cell / 2 * 418;
    Some(html! {
        <svg class="scene-background ally-setting" data-external-contact="true" data-ally-unit={unit.to_owned()} viewBox="0 0 627 418" preserveAspectRatio="xMidYMid meet">
            <svg viewBox={format!("{x} {y} 627 418")} width="627" height="418" overflow="hidden">
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/ally-settings-{variant}-20260915.png"))} width="1254" height="1254"/>
            </svg>
        </svg>
    })
}
