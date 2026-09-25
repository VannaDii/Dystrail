//! Completed barter cells retain the source camera and never imply an uncommitted trade.
use super::SceneStage;
use yew::prelude::*;

fn context(stage: &SceneStage) -> Option<(&str, &str, u32, u32)> {
    let SceneStage::EncounterOutcome { unit, choice: 0 } = stage else { return None; };
    let (family, variant) = unit.rsplit_once('-')?;
    let row = match family {
        "ACT-BARTERTIRE" => 0,
        "ACT-BARTERBATTERY" => 1,
        "ACT-BARTERSUPPLIES" => 2,
        _ => return None,
    };
    let boundaries = match variant {
        "A" => [0, 340, 683, 1024],
        "B" => [0, 341, 680, 1024],
        "C" => [0, 341, 684, 1024],
        _ => return None,
    };
    Some((unit, variant, boundaries[row] + 4, boundaries[row + 1] - boundaries[row] - 8))
}

pub fn aspect(stage: &SceneStage) -> Option<String> {
    context(stage).map(|(_, _, _, height)| format!("760 / {height}"))
}

pub fn render(stage: &SceneStage) -> Option<Html> {
    let (unit, variant, y, height) = context(stage)?;
    Some(html! {
        <svg class="scene-background barter-setting" data-barter-unit={unit.to_owned()} data-trade-completed="true"
            viewBox={format!("0 0 760 {height}")} preserveAspectRatio="xMidYMid meet">
            <svg viewBox={format!("772 {y} 760 {height}")} width="760" height={height.to_string()} overflow="hidden">
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/barter-{}.png",variant.to_ascii_lowercase()))} width="1536" height="1024"/>
            </svg>
        </svg>
    })
}
