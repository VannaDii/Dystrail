//! Teach-in scenes preserve the offer and either of the two committed choices.
use yew::prelude::*;

pub fn render(unit: &str, choice: Option<usize>) -> Option<Html> {
    let variant = match unit {
        "ENC-C06-A" => "a",
        "ENC-C06-B" => "b",
        "ENC-C06-C" => "c",
        _ => return None,
    };
    let selected = choice.filter(|c| *c < 2);
    // Cell three is an unused offer reference, never an invented third choice.
    let cell = selected.map_or(0, |c| c + 1);
    let (x, y) = ((cell % 2) * 768 + 4, (cell / 2) * 512 + 4);
    let atlas = format!("road-c06-{variant}-20260914");
    let clip = format!("teachin-frame-{unit}-{cell}");
    Some(html! {
        <svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas.clone()} data-cell={cell.to_string()} viewBox={format!("{x} {y} 760 504")} preserveAspectRatio="xMidYMid meet">
            <defs><clipPath id={clip.clone()}><rect x={x.to_string()} y={y.to_string()} width="760" height="504"/></clipPath></defs>
            <g clip-path={format!("url(#{clip})")}>
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024"/>
                if selected.is_none() && variant != "b" {
                    <g class="road-prop-lettering" fill="#342b20" font-family="sans-serif" font-size="18" font-weight="700" text-anchor="middle">
                        if variant == "a" {
                            <text x="381" y="363" textLength="120" lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line1"))}</text>
                        } else {
                            <text x="316" y="268" textLength="94" lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line1"))}</text>
                            <text x="316" y="294" textLength="94" lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line2"))}</text>
                        }
                    </g>
                }
            </g>
        </svg>
    })
}
