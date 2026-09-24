//! Only retained cells whose premise still matches approved care copy.
use super::{Props, SceneStage};
use yew::prelude::*;

pub fn context(stage: &SceneStage) -> Option<(&str, u8, &str)> {
    let SceneStage::CareIncident { unit, .. } = stage else {
        return None;
    };
    let (family, variant) = unit.rsplit_once('-')?;
    if !matches!(variant, "A" | "B" | "C") {
        return None;
    }
    let cell = match family {
        "CARE-01" => 0,
        "CARE-02" => 1,
        "CARE-03" => 2,
        _ => return None,
    };
    Some((unit, cell, variant))
}

pub fn indoors(stage: &SceneStage) -> Option<bool> {
    context(stage).map(|(_, cell, _)| cell == 2)
}

pub fn render(p: &Props) -> Option<Html> {
    let (unit, cell, variant) = context(&p.stage)?;
    let x = u32::from(cell % 2) * 768;
    let y = u32::from(cell / 2) * 512;
    Some(html! {
        <svg class="scene-background care-setting" data-care-setting={unit.to_owned()} viewBox="0 0 768 512" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
            <svg viewBox={format!("{x} {y} 768 512")} width="768" height="512" overflow="hidden">
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/care-settings-{}-20260914.png",variant.to_ascii_lowercase()))} width="1536" height="2048"/>
            </svg>
        </svg>
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_premises_cannot_select_retained_cells() {
        for n in 1..=8 {
            for variant in ["A", "B", "C"] {
                let stage = SceneStage::CareIncident {
                    unit: format!("CARE-{n:02}-{variant}"),
                    persona: "organizer".into(),
                };
                assert_eq!(context(&stage).is_some(), n <= 3);
            }
        }
    }
}
