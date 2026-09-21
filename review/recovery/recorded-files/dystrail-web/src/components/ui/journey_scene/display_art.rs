//! Picnic, product-origin and shade displays follow the committed C04 choice.
use yew::prelude::*;

pub fn render(unit: &str, choice: Option<usize>) -> Option<Html> {
    let variant = match unit {
        "ENC-C04-A" => "a",
        "ENC-C04-B" => "b",
        "ENC-C04-C" => "c",
        _ => return None,
    };
    let cell = choice.filter(|c| *c < 3).map_or(0, |c| c + 1);
    let dx = (cell % 2) * 768;
    let dy = (cell / 2) * 512;
    let (x, y) = (dx + 4, dy + 4);
    let atlas = format!("road-c04-{variant}-20260914");
    let clip = format!("display-frame-{unit}-{cell}");
    Some(html! {
        <svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas.clone()} data-cell={cell.to_string()} viewBox={format!("{x} {y} 760 504")} preserveAspectRatio="xMidYMid meet">
            <defs><clipPath id={clip.clone()}><rect x={x.to_string()} y={y.to_string()} width="760" height="504"/></clipPath></defs>
            <g clip-path={format!("url(#{clip})")}>
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024"/>
                if variant == "c" {
                    <g class="road-prop-lettering" fill="#342b20" font-family="sans-serif" font-size="23" font-weight="700" text-anchor="middle">
                        <text x={(146+dx).to_string()} y={(357+dy).to_string()} textLength="210" lengthAdjust="spacingAndGlyphs">{crate::i18n::t("visual_copy.ENC-C04-C.overlay_line1")}</text>
                        <text x={(146+dx).to_string()} y={(386+dy).to_string()} textLength="180" lengthAdjust="spacingAndGlyphs">{crate::i18n::t("visual_copy.ENC-C04-C.overlay_line2")}</text>
                    </g>
                }
            </g>
        </svg>
    })
}
