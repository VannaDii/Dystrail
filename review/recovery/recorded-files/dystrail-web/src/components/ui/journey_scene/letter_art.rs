//! Letter offers and their actual saved, recorded or shared-food outcomes.
use yew::prelude::*;

fn windows(variant: &str, cell: usize) -> Html {
    // Visible glass only: leave the foreground people, frames and furniture dry.
    let glass = match variant {
        "a" => "M172 4H277V42H172Z M339 4H438V30L421 52V119H350V62H339Z M506 4H613V132H586V75L556 35H506Z",
        "c" => "M4 32L32 48V166L4 179Z M41 53L83 75V163L41 172Z",
        _ => return Html::default(),
    };
    let clip = format!("letter-window-{variant}-{cell}");
    let rain = format!("letter-rain-{variant}-{cell}");
    let snow = format!("letter-snow-{variant}-{cell}");
    html! {
        <g class="letter-window-layer" transform={format!("translate({} {})", (cell%2)*768, (cell/2)*512)}>
            <defs>
                <clipPath id={clip.clone()}><path d={glass}/></clipPath>
                <pattern id={rain.clone()} width="27" height="37" patternUnits="userSpaceOnUse"><path d="M8 4L4 14M23 23L19 33" stroke="#d8e6ef" stroke-width="2"/></pattern>
                <pattern id={snow.clone()} width="31" height="29" patternUnits="userSpaceOnUse"><path d="M5 5H8V8H5Z M22 19H24V21H22Z" fill="#eef3f4"/></pattern>
            </defs>
            <g clip-path={format!("url(#{clip})")}>
                <rect class="letter-window-shade" width="768" height="512"/>
                <rect class="letter-window-rain" width="768" height="512" fill={format!("url(#{rain})")}/>
                <rect class="letter-window-snow" width="768" height="512" fill={format!("url(#{snow})")}/>
            </g>
        </g>
    }
}

pub fn render(unit: &str, choice: Option<usize>) -> Option<Html> {
    let (variant, label_x, label_y, width) = match unit {
        "ENC-C05-A" => ("a", 267, 280, 130),
        "ENC-C05-B" => ("b", 284, 226, 135),
        "ENC-C05-C" => ("c", 371, 342, 135),
        _ => return None,
    };
    let selected = choice.filter(|c| *c < 3);
    let cell = selected.map_or(0, |c| c + 1);
    let (x, y) = ((cell % 2) * 768 + 4, (cell / 2) * 512 + 4);
    let atlas = format!("road-c05-{variant}-20260914");
    let clip = format!("letter-frame-{unit}-{cell}");
    Some(html! {
        <svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas.clone()} data-cell={cell.to_string()} viewBox={format!("{x} {y} 760 504")} preserveAspectRatio="xMidYMid meet">
            <defs><clipPath id={clip.clone()}><rect x={x.to_string()} y={y.to_string()} width="760" height="504"/></clipPath></defs>
            <g clip-path={format!("url(#{clip})")}>
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024"/>
                {windows(variant, cell)}
                if selected.is_none() {
                    <g class="road-prop-lettering" fill="#342b20" font-family="sans-serif" font-size="18" font-weight="700" text-anchor="middle">
                        <text x={label_x.to_string()} y={label_y.to_string()} textLength={width.to_string()} lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line1"))}</text>
                        <text x={label_x.to_string()} y={(label_y+24).to_string()} textLength={width.to_string()} lengthAdjust="spacingAndGlyphs">{crate::i18n::t(&format!("visual_copy.{unit}.overlay_line2"))}</text>
                    </g>
                }
            </g>
        </svg>
    })
}
