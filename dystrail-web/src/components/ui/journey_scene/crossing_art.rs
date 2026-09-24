//! Native scene masks retain authored people/props over the actual route region.
//! These SVG crops never rewrite raster files or affect simulation state.
use super::{Props, Region, SceneStage, reviewed_road};
use yew::prelude::*;

pub fn supported(unit: &str) -> bool {
    matches!(unit, "CROSS-02C-A" | "CROSS-02D-A")
}

pub fn render(p: &Props) -> Option<Html> {
    let SceneStage::Crossing { unit, passed } = &p.stage else { return None; };
    let asset = match unit.as_str() {
        "CROSS-02D-A" => "crossing-privacy-v2".to_owned(),
        "CROSS-02C-A" => format!("crossing-workers-{}-v2", if *passed { "raised" } else { "lowered" }),
        _ => return None,
    };
    let region = reviewed_road(p.region.unwrap_or(Region::Heartland), p.day, p.road_asset.as_deref());
    let props_id = format!("crossing-props-{unit}");
    let land_id = format!("crossing-land-{unit}");
    let road_id = format!("crossing-road-{unit}");
    let workers = unit == "CROSS-02C-A";
    let privacy = unit == "CROSS-02D-A";
    Some(html! {
        <svg class="scene-background crossing-layered-art" viewBox="0 0 1536 1024" preserveAspectRatio="xMidYMid meet" data-route-art={region}>
            <defs>
                <clipPath id={land_id.clone()}><path d="M0 0H1536V578H0Z" /></clipPath>
                <clipPath id={road_id.clone()}><path d="M0 576H1536V1024H0Z" /></clipPath>
                <clipPath id={props_id.clone()}>
                    if workers {
                        <path d="M638 272H1030V322L1000 345H978V542H984V561H691V545H659V468H713V345H669L638 323Z" />
                        // Bodies, heads and individual legs keep the regional sky
                        // visible between limbs; the beam is a separate object.
                        <path d="M610 390L622 386L638 393L645 411L637 425H612L603 415L605 400Z M605 421L642 419L657 442L652 480H598L594 444Z M599 478H625L621 542L627 557L593 562L592 551L598 537Z M630 478H651L650 542L657 553V561H638L635 548Z" />
                        <path d="M1025 393L1037 388L1051 394L1057 408L1051 423H1028L1020 413Z M1022 420H1053L1066 442L1063 481H1010L1008 446Z M1011 477H1033L1031 542L1037 558L1007 562L1008 548Z M1038 477H1064L1063 540L1076 553V562H1046L1040 548Z" />
                        if *passed {
                            <path d="M569 353H1097V376H569Z M586 344H596V378L605 410L613 425L604 437L590 414Z M654 369H665L667 413L655 437L643 425L653 408Z M1000 369H1015L1012 405L1026 425L1014 438L1004 414Z M1059 370H1075L1072 416L1061 435L1050 425L1058 405Z" />
                        }else{
                            <path d="M563 466H1095V489H563Z M593 425L608 432L599 460L602 473H586L583 457Z M643 427L655 436L660 460L657 473H643L644 457Z M1013 425L1027 433L1020 459L1022 474H1007L1006 453Z M1054 426L1069 440L1077 464L1074 473H1061L1062 455Z" />
                        }
                    }else{
                        <path d="M674 291L695 275H1036V301H1001V542H1008V561H699V545H660V468H714V328H677Z" />
                    }
                    if privacy {
                        <path d="M495 320L513 318L548 328L546 349H519V521H528V560H490V520H501V351H495Z M1012 244L1059 252L1056 270H1027V276H999V262H1009Z" />
                        <path d="M578 398L588 390L606 386L618 396L622 415L615 429L630 445L643 437L660 444L653 452L639 458L621 451L618 483L620 540L627 552L625 567H579L576 554L580 541L578 490L566 473L559 461L560 435L569 427Z" />
                    }
                </clipPath>
            </defs>
            <image href={crate::paths::asset_path(&format!("static/img/journey/{region}.png"))} x="0" y="-98" width="1536" height="1024" clip-path={format!("url(#{land_id})")} />
            <image class="crossing-fork" href={crate::paths::asset_path("static/img/scenes-v2/crossing-fork-v2.png")} width="1536" height="1024" clip-path={format!("url(#{road_id})")} />
            <image class="crossing-authored-outcome" data-crossing-unit={unit.clone()} href={crate::paths::asset_path(&format!("static/img/scenes-v2/{asset}.png"))} width="1536" height="1024" clip-path={format!("url(#{props_id})")} />
            if workers {
                <foreignObject x="650" y="279" width="365" height="34">
                    <div xmlns="http://www.w3.org/1999/xhtml" dir="auto" style="height:100%;display:flex;align-items:center;justify-content:center;text-align:center;font:bold 19px/1 sans-serif;color:#261e14">{crate::i18n::t(&format!("encounter_copy.{unit}.name"))}</div>
                </foreignObject>
            }
        </svg>
    })
}
