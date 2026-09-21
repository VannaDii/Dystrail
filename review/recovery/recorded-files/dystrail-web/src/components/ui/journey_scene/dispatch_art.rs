//! Civic displays: offer, donated materials, shared meal, or checked evidence.
use yew::prelude::*;

pub fn render(unit: &str, choice: Option<usize>) -> Option<Html> {
    let (variant, label_x, line_y, width, font_size) = match unit {
        "ENC-C08-A" => ("a", 459, [64, 85], 270, 18),
        "ENC-C08-B" => ("b", 120, [218, 232], 64, 12),
        "ENC-C08-C" => ("c", 91, [300, 330], 110, 20),
        _ => return None,
    };
    let selected = choice.filter(|c| *c < 3);
    let cell = selected.map_or(0, |c| c + 1);
    let (x, y) = ((cell % 2) * 768 + 8, (cell / 2) * 512 + 8);
    let atlas = format!("road-c08-{variant}-20260914");
    let clip = format!("dispatch-frame-{unit}-{cell}");
    Some(html! {
        <svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas.clone()} data-cell={cell.to_string()} viewBox={format!("{x} {y} 752 496")} preserveAspectRatio="xMidYMid meet">
            <defs><clipPath id={clip.clone()}><rect x={x.to_string()} y={y.to_string()} width="752" height="496"/></clipPath></defs>
            <g clip-path={format!("url(#{clip})")}>
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024"/>
                if selected.is_none() {
                    <g class="road-prop-lettering" fill="#342b20" font-family="sans-serif" font-size={font_size.to_string()} font-weight="700" text-anchor="middle">
                        {for line_y.iter().enumerate().map(|(i,y)|html!{<text x={label_x.to_string()} y={y.to_string()} textLength={width.to_string()} lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line{}",i+1))}</text>})}
                    </g>
                }
            </g>
        </svg>
    })
}
