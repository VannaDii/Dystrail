//! Secular pantry scenes show the actual choice; lettering stays localized.
use yew::prelude::*;

pub fn render(unit: &str, choice: Option<usize>) -> Option<Html> {
    let row = match unit {
        "ENC-C02-A" => 0,
        "ENC-C02-B" => 1,
        "ENC-C02-C" => 2,
        _ => return None,
    };
    let (atlas, right) = match choice {
        Some(0) => ("road-c02-offer-meal-20260914", true),
        Some(1) => ("road-c02-record-donate-20260914", false),
        Some(2) => ("road-c02-record-donate-20260914", true),
        _ => ("road-c02-offer-meal-20260914", false),
    };
    let bounds = [0, 374, 690, 1024];
    let dx = if right { 768 } else { 0 };
    let x = dx + 4;
    let y = bounds[row] + 4;
    let h = bounds[row + 1] - bounds[row] - 8;
    let (label_x, label_y, label_w, line_gap) = match row {
        0 => (346, 326, 242, 27),
        1 if choice == Some(2) => (145, 650, 88, 21),
        1 => (174, 650, 140, 21),
        _ => (88, 897, 121, 22),
    };
    let clip = format!("pantry-frame-{unit}-{}", choice.map_or("offer".into(), |c| c.to_string()));
    Some(html! {
        <svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas} data-cell={(row*2+usize::from(right)).to_string()} viewBox={format!("{x} {y} 760 {h}")} preserveAspectRatio="xMidYMid meet">
            <defs><clipPath id={clip.clone()}><rect x={x.to_string()} y={y.to_string()} width="760" height={h.to_string()}/></clipPath></defs>
            <g clip-path={format!("url(#{clip})")}>
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024"/>
                <g class="road-prop-lettering" fill="#342b20" font-family="sans-serif" font-size={if row == 0 {"25"} else {"20"}} font-weight="700" text-anchor="middle">
                    <text x={(label_x+dx).to_string()} y={label_y.to_string()} textLength={label_w.to_string()} lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line1"))}</text>
                    <text x={(label_x+dx).to_string()} y={(label_y+line_gap).to_string()} textLength={label_w.to_string()} lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line2"))}</text>
                </g>
            </g>
        </svg>
    })
}
