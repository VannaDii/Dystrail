//! Retained activity outcomes appear only after the matching work is committed.
use super::SceneStage;
use yew::prelude::*;

pub fn supported(stage: &SceneStage) -> bool {
    matches!(stage, SceneStage::EncounterOutcome { unit, choice: 0 } if matches!(unit.as_str(), "ACT-FOODWORK-A" | "ACT-CASHWORK-A"))
}

pub fn render(stage: &SceneStage) -> Option<Html> {
    if !supported(stage) { return None; }
    let SceneStage::EncounterOutcome { unit, .. } = stage else { return None; };
    let pantry = unit == "ACT-FOODWORK-A";
    Some(html! {
        <svg class="scene-background activity-setting" data-activity-unit={unit.clone()}
            data-activity-completed="true" viewBox={if pantry { "772 4 760 248" } else { "772 260 760 248" }} preserveAspectRatio="xMidYMid meet" aria-hidden="true">
            <image href={crate::paths::asset_path("static/img/scenes-v2/activity-work-retained.png")} width="1536" height="1024"/>
            // Keep carton closures plain; no insignia or lettering is part of this scene.
            if pantry { <g fill="#23455a" data-plain-carton-tops="true">
                <path d="M986 172H1002V180H986Z M1010 168H1027V177H1010Z M1035 162H1050V172H1035Z M1062 166H1079V178H1062Z"/>
            </g> } else {
                // An empty, unlettered glass jar on the cleaned counter. Its stepped
                // silhouette follows the retained art's pixel edges, without raster text.
                <g data-empty-tip-jar="true" transform="translate(360 0)" shape-rendering="crispEdges">
                    <path d="M1095 489H1136V493H1095Z" fill="#142b35" opacity="0.45"/>
                    <path d="M1102 438H1128V444H1132V485H1128V489H1102V485H1098V444H1102Z" fill="#aec8c9" fill-opacity="0.22" stroke="#233944" stroke-width="3"/>
                    <path d="M1103 437H1127V442H1103Z" fill="#b9cdd0" stroke="#344852" stroke-width="2"/>
                    <path d="M1104 449V479H1108V449Z M1105 482H1126V485H1105Z" fill="#e0e7df" opacity="0.65"/>
                </g>
            }
        </svg>
    })
}
