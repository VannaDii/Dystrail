//! Reviewed outside-contact cells, never members of the traveling party.
use super::SceneStage;
use yew::prelude::*;

pub fn coordinates(unit: &str) -> Option<(u32, char)> {
    match unit {
        "ALLY-01-A" => Some((0, 'a')),
        "ALLY-05-B" => Some((4, 'b')),
        "ALLY-02-A" => Some((1, 'a')),
        "ALLY-02-C" => Some((1, 'c')),
        "ALLY-04-B" => Some((3, 'b')),
        "ALLY-05-A" => Some((4, 'a')),
        _ => None,
    }
}

pub fn context(stage: &SceneStage) -> Option<(&str, u32, char)> {
    let SceneStage::Encounter(unit) = stage else { return None; };
    let (cell, variant) = coordinates(unit)?;
    Some((unit, cell, variant))
}

pub fn render(stage: &SceneStage) -> Option<Html> {
    let (unit, cell, variant) = context(stage)?;
    let x = cell % 2 * 627;
    let y = cell / 2 * 418;
    let labels = match unit {
        "ALLY-01-A" => html! {
            <g data-ally-labels="referral">
                <rect x="457" y="40" width="133" height="87" fill="#fff8ea"/>
                <foreignObject x="464" y="45" width="119" height="77">
                    <div xmlns="http://www.w3.org/1999/xhtml" dir="auto" style="color:#242b34;font-family:system-ui,sans-serif;font-weight:700;text-align:center;line-height:1.1;overflow-wrap:anywhere;hyphens:auto;">
                        <div style="font-size:22px;margin-bottom:5px;">{"MAGA"}</div>
                        <div class="ally-prop-label" style="font-size:14px;">{crate::i18n::t("ally_art.affiliate_link")}</div>
                    </div>
                </foreignObject>
            </g>
        },
        "ALLY-05-B" => html! {
            <g data-ally-labels="milk" transform="translate(182 223) matrix(1 -0.06 0.25 1 0 0)">
                <rect width="42" height="97" fill="#e2f0fa"/>
                <foreignObject x="2" y="8" width="38" height="80">
                    <div xmlns="http://www.w3.org/1999/xhtml" dir="auto" class="ally-prop-label" style="color:#182d3d;font-family:system-ui,sans-serif;font-size:8px;font-weight:700;line-height:1.1;text-align:center;overflow-wrap:anywhere;hyphens:auto;">{crate::i18n::t("ally_art.fda_operative")}</div>
                </foreignObject>
            </g>
        },
        _ => Html::default(),
    };
    Some(html! {
        <svg class="scene-background ally-setting" data-external-contact="true" data-ally-unit={unit.to_owned()} viewBox="0 0 627 418" preserveAspectRatio="xMidYMid meet">
            <svg viewBox={format!("{x} {y} 627 418")} width="627" height="418" overflow="hidden">
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/ally-settings-{variant}-20260915.png"))} width="1254" height="1254"/>
            </svg>
            {labels}
        </svg>
    })
}
