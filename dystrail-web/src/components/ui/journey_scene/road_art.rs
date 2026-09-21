//! Authored road decisions retain the selected unit and actual committed choice.
use super::SceneStage;
use yew::prelude::*;

pub fn context(stage: &SceneStage) -> Option<(&str, Option<usize>)> {
    let (unit, choice) = match stage {
        SceneStage::Encounter(unit) => (unit.as_str(), None),
        SceneStage::EncounterOutcome { unit, choice } => (unit.as_str(), Some(*choice)),
        _ => return None,
    };
    supported(unit).then_some((unit, choice))
}

pub fn supported(unit: &str) -> bool {
    matches!(
        unit,
        "ENC-C01-A" | "ENC-C01-B" | "ENC-C01-C" | "ENC-C03-A" | "ENC-C03-B" | "ENC-C03-C"
    )
}

pub fn is_indoors(unit: &str) -> bool {
    matches!(unit, "ENC-C01-B" | "ENC-C03-A" | "ENC-C03-C")
}

pub fn aftermath(unit: String, choice: usize) -> SceneStage {
    if supported(&unit) {
        SceneStage::EncounterOutcome { unit, choice }
    } else {
        SceneStage::Encounter(unit)
    }
}

pub fn label_description(stage: &SceneStage) -> Option<String> {
    let (unit, _) = context(stage)?;
    let count = match unit {
        "ENC-C01-B" | "ENC-C03-A" | "ENC-C03-B" | "ENC-C03-C" => 1,
        "ENC-C01-C" => 2,
        _ => return None,
    };
    Some(
        (0..count)
            .map(|i| crate::i18n::t(&format!("encounter_copy.{unit}.overlay_{i}")))
            .collect::<Vec<_>>()
            .join(" · "),
    )
}

fn labels(unit: &str, worked: bool) -> Html {
    let dx = if worked { 768 } else { 0 };
    if unit == "ENC-C01-B" {
        let (first, second) = match crate::i18n::current_lang().as_str() {
            "es" => ("PIEZA NO", "RECONOCIDA"),
            "it" => ("RICAMBIO NON", "RICONOSCIUTO"),
            "ar" => ("قطعة غير", "معترف بها"),
            _ => ("PART NOT", "RECOGNIZED"),
        };
        return html! {<g class="road-prop-lettering" fill="#fff1cc" font-family="sans-serif" font-size="26" font-weight="700" text-anchor="middle">
            <text x={(640+dx).to_string()} y="374">{first}</text>
            <text x={(640+dx).to_string()} y="399">{second}</text>
        </g>};
    }
    if unit != "ENC-C01-C" {
        return Html::default();
    }
    let (left, right) = if worked { (143, 626) } else { (143, 624) };
    html! {<g class="road-prop-lettering" fill="#312a20" font-size="26" font-weight="700" text-anchor="middle" direction="ltr">
        <text font-family="Arial, sans-serif" x={(left+dx).to_string()} y="942">{crate::i18n::t("encounter_copy.ENC-C01-C.overlay_0")}</text>
        <text font-size="22" font-family="Georgia, serif" x={(right+dx).to_string()} y="930">{"TIMES NEW"}</text>
        <text font-size="22" font-family="Georgia, serif" x={(right+dx).to_string()} y="957">{"ROMAN"}</text>
    </g>}
}

pub fn render(stage: &SceneStage) -> Option<Html> {
    let (unit, choice) = context(stage)?;
    if unit.starts_with("ENC-C03-") {
        return Some(render_c03(unit, choice));
    }
    let row = match unit {
        "ENC-C01-A" => 0,
        "ENC-C01-B" | "ENC-C03-A" | "ENC-C03-B" | "ENC-C03-C" => 1,
        _ => 2,
    };
    // Photographing changes evidence, not the site. Only the actual help choice
    // uses the worked frame. Legacy outcomes without a choice keep the offer.
    let worked = choice == Some(0);
    let bounds = [0, 341, 656, 1024];
    let x = if worked { 772 } else { 4 };
    let y = bounds[row] + 4;
    let h = bounds[row + 1] - bounds[row] - 8;
    let clip = format!(
        "road-frame-{unit}-{}",
        choice.map_or("offer".into(), |c| c.to_string())
    );
    Some(
        html! {<svg class="scene-background scene-atlas" aria-hidden="true" data-atlas="road-c01-20260914" data-cell={(row*2+usize::from(worked)).to_string()} viewBox={format!("{x} {y} 760 {h}")} preserveAspectRatio="xMidYMid meet">
            <defs><clipPath id={clip.clone()}><rect x={x.to_string()} y={y.to_string()} width="760" height={h.to_string()}/></clipPath></defs>
            <g clip-path={format!("url(#{clip})")}><image href={crate::paths::asset_path("static/img/scenes-v2/road-c01-20260914.png")} width="1536" height="1024"/>{labels(unit,worked)}</g>
        </svg>},
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_committed_reviewed_choices_gain_an_outcome_stage() {
        for unit in ["ENC-C01-A", "ENC-C01-B", "ENC-C01-C"] {
            for choice in 0..2 {
                let stage = aftermath(unit.into(), choice);
                let restored: SceneStage =
                    serde_json::from_str(&serde_json::to_string(&stage).unwrap()).unwrap();
                assert_eq!(context(&restored), Some((unit, Some(choice))));
            }
            let old: SceneStage =
                serde_json::from_str(&format!("{{\"Encounter\":\"{unit}\"}}")).unwrap();
            assert_eq!(context(&old), Some((unit, None)));
        }
        assert_eq!(
            aftermath("unknown".into(), 0),
            SceneStage::Encounter("unknown".into())
        );
    }
}

pub fn aspect(stage: &SceneStage) -> Option<&'static str> {
    let (unit, _) = context(stage)?;
    Some(match unit {
        "ENC-C01-A" => "760 / 333",
        "ENC-C01-B" => "760 / 307",
        "ENC-C03-A" | "ENC-C03-B" | "ENC-C03-C" => "760 / 504",
        _ => "760 / 360",
    })
}

/// Offer followed by the three actual committed outcomes, in atlas row order.
fn render_c03(unit: &str, choice: Option<usize>) -> Html {
    let cell = choice.filter(|c| *c < 3).map_or(0, |c| c + 1);
    let dx = (cell % 2) * 768;
    let dy = (cell / 2) * 512;
    let atlas = format!("road-{}-20260914", unit[4..].to_ascii_lowercase());
    let clip = format!("road-frame-{unit}-{cell}");
    let label = crate::i18n::t(&format!("encounter_copy.{unit}.overlay_0"));
    // Match each retained blank prop face; never stretch typography or art.
    let (x, y, w, h) = match (unit, cell) {
        ("ENC-C03-A", 0 | 1) => (284, 332, 252, 104),
        ("ENC-C03-A", _) => (376, 854 - 512, 240, 92),
        ("ENC-C03-B", _) => (273, 91, 244, 65),
        ("ENC-C03-C", 3) => (326, 668 - 512, 96, 40),
        ("ENC-C03-C", _) => (291, 208, 140, 50),
        _ => unreachable!(),
    };
    html! {<svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas.clone()} data-cell={cell.to_string()} viewBox={format!("{} {} 760 504",dx+4,dy+4)} preserveAspectRatio="xMidYMid meet">
        <defs><clipPath id={clip.clone()}><rect x={(dx+4).to_string()} y={(dy+4).to_string()} width="760" height="504"/></clipPath></defs>
        <g clip-path={format!("url(#{clip})")}>
            <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024"/>
            <foreignObject class="road-prop-lettering" x={(dx+x).to_string()} y={(dy+y).to_string()} width={w.to_string()} height={h.to_string()}>
                <div xmlns="http://www.w3.org/1999/xhtml" dir="auto" style={format!("height:100%;display:flex;align-items:center;justify-content:center;text-align:center;color:#312a20;font:bold {}px/1.05 sans-serif;overflow-wrap:anywhere",if unit=="ENC-C03-C" {14} else {24})}>{label}</div>
            </foreignObject>
        </g>
    </svg>}
}
