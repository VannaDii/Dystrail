//! One offer and three committed outcomes for each fundraiser satire variant.
use yew::prelude::*;

pub fn render(unit: &str, choice: Option<usize>) -> Option<Html> {
    let variant = match unit {
        "ENC-C03-A" => "a",
        "ENC-C03-B" => "b",
        "ENC-C03-C" => "c",
        _ => return None,
    };
    // Unknown/legacy choices retain the offer rather than inventing a donation.
    let cell = choice.filter(|c| *c < 3).map_or(0, |c| c + 1);
    let dx = (cell % 2) * 768;
    let dy = (cell / 2) * 512;
    let (label_x, label_y, label_w, gap) = match (variant, cell) {
        ("a", 2) => (464, 372, 190, 28),
        ("a", 3) => (416, 372, 236, 28),
        ("a", _) => (414, 369, 230, 28),
        ("b", _) => (394, 116, 224, 24),
        ("c", 3) => (365, 179, 114, 18),
        _ => (362, 231, 132, 20),
    };
    let x = dx + 4;
    let y = dy + 4;
    let atlas = format!("road-c03-{variant}-20260914");
    let clip = format!("fundraiser-frame-{unit}-{cell}");
    Some(html! {
        <svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas.clone()} data-cell={cell.to_string()} viewBox={format!("{x} {y} 760 504")} preserveAspectRatio="xMidYMid meet">
            <defs><clipPath id={clip.clone()}><rect x={x.to_string()} y={y.to_string()} width="760" height="504"/></clipPath></defs>
            <g clip-path={format!("url(#{clip})")}>
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024"/>
                <g class="road-prop-lettering" fill="#342b20" font-family="sans-serif" font-size={if variant == "c" {"17"} else {"23"}} font-weight="700" text-anchor="middle">
                    <text x={(label_x+dx).to_string()} y={(label_y+dy).to_string()} textLength={label_w.to_string()} lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line1"))}</text>
                    <text x={(label_x+dx).to_string()} y={(label_y+dy+gap).to_string()} textLength={label_w.to_string()} lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line2"))}</text>
                    if variant == "a" && cell == 2 {
                        <rect x="146" y="748" width="102" height="26" rx="2" fill="#fff0c9"/>
                        <text x="197" y="766" font-size="12" textLength="94" lengthAdjust="spacingAndGlyphs">{crate::i18n::t("visual_copy.ENC-C03-A.overlay_1")}</text>
                    }
                </g>
            </g>
        </svg>
    })
}
