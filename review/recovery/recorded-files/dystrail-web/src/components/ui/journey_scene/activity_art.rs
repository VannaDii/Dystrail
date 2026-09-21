//! Explicit source bounds keep independently authored before/after rows intact.
use yew::prelude::*;

pub fn setting_name(unit: &str) -> Option<&'static str> {
    let (family, variant) = unit.rsplit_once('-')?;
    if !matches!(variant, "A" | "B" | "C") { return None; }
    match family {
        "ACT-FORAGE" => Some("activity-forage"),
        "ACT-GLEAN" => Some("activity-glean"),
        "ACT-FOODWORK" => Some("activity-foodwork"),
        "ACT-CASHWORK" => Some("activity-cashwork"),
        "ACT-BARTERTIRE" => Some("activity-bartertire"),
        "ACT-BARTERBATTERY" => Some("activity-barterbattery"),
        "ACT-BARTERSUPPLIES" => Some("activity-bartersupplies"),
        "ACT-REST" => Some("activity-rest"),
        _ => None,
    }
}

pub fn indoors(unit: &str) -> bool {
    setting_name(unit) == Some("activity-foodwork")
        || (setting_name(unit) == Some("activity-cashwork") && unit != "ACT-CASHWORK-C")
}

pub fn render(unit: &str, completed: bool) -> Option<Html> {
    let setting = setting_name(unit)?;
    if setting == "activity-rest" { return None; }
    let row = match setting {
        "activity-forage" => 0,
        "activity-glean" => 1,
        "activity-foodwork" => 2,
        "activity-bartertire" => 0,
        "activity-barterbattery" => 1,
        "activity-bartersupplies" => 2,
        _ => 3,
    };
    let (atlas, boundaries) = match (setting.starts_with("activity-barter"), unit.rsplit_once('-')?.1) {
        (true, "A") => ("barter-a", [0, 340, 683, 1024, 1024]),
        (true, "B") => ("barter-b", [0, 341, 680, 1024, 1024]),
        (true, _) => ("barter-c", [0, 341, 684, 1024, 1024]),
        (false, "A") => ("activities-a", [0, 242, 489, 721, 1024]),
        (false, "B") => ("activities-b", [0, 256, 510, 733, 1024]),
        (false, _) => ("activities-c", [0, 256, 512, 737, 1024]),
    };
    // Four source pixels of inset exclude seams without resizing or filtering art.
    let x = if completed { 772 } else { 4 };
    let y = boundaries[row] + 4;
    let height = boundaries[row + 1] - boundaries[row] - 8;
    let clip_id = format!("frame-{unit}-{completed}");
    Some(html! {
        <svg class="scene-background scene-atlas" aria-hidden="true" data-atlas={atlas}
            data-cell={(row * 2 + usize::from(completed)).to_string()}
            viewBox={format!("{x} {y} 760 {height}")} preserveAspectRatio="xMidYMid meet">
            <defs><clipPath id={clip_id.clone()}><rect x={x.to_string()} y={y.to_string()} width="760" height={height.to_string()} /></clipPath></defs>
            <g clip-path={format!("url(#{clip_id})")}><image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024" /></g>
        </svg>
    })
}
