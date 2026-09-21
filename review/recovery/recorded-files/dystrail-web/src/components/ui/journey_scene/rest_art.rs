//! Rest props are composited over the real road, independently of the living crew.
use yew::prelude::*;

pub fn render(unit: &str, completed: bool) -> Html {
    let row = match unit { "ACT-REST-B" => 1, "ACT-REST-C" => 2, _ => 0 };
    let x = if completed {825} else {260};
    let y = [30,350,670][row];
    let clip = format!("rest-frame-{row}-{completed}");
    html! {<svg class="rest-props" data-cell={(row*2+usize::from(completed)).to_string()} viewBox={format!("{x} {y} 470 310")} preserveAspectRatio="xMidYMid meet" aria-hidden="true">
        <defs><clipPath id={clip.clone()}><rect x={x.to_string()} y={y.to_string()} width="470" height="310" /></clipPath></defs>
        <g clip-path={format!("url(#{clip})")}><image href={crate::paths::asset_path("static/img/scenes-v2/rest-props.png")} width="1536" height="1024" /></g>
    </svg>}
}
