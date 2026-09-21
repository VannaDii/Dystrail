//! Editable lettering for authored props; kept outside the tinted raster artwork.
use super::SceneStage;
use yew::prelude::*;

pub fn render(stage: &SceneStage) -> Html {
    let SceneStage::Encounter(unit) = stage else { return Html::default(); };
    let count = match unit.as_str() {
        "ENC-C01-B" => 1,
        "ENC-C01-C" => 2,
        _ => return Html::default(),
    };
    html! {<div class="scene-annotations" data-unit={unit.clone()}>
        {for (0..count).map(|i|html! {<span class="scene-prop-lettering" dir="auto">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_{i}"))}</span>})}
    </div>}
}
