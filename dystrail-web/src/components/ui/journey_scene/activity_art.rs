//! Retained activity outcomes appear only after the matching work is committed.
use super::SceneStage;
use yew::prelude::*;

pub fn supported(stage: &SceneStage) -> bool {
    matches!(stage, SceneStage::EncounterOutcome { unit, choice: 0 } if unit == "ACT-FOODWORK-A")
}

pub fn render(stage: &SceneStage) -> Option<Html> {
    if !supported(stage) { return None; }
    Some(html! {
        <svg class="scene-background activity-setting" data-activity-unit="ACT-FOODWORK-A"
            data-activity-completed="true" viewBox="772 4 760 248" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
            <image href={crate::paths::asset_path("static/img/scenes-v2/activity-work-retained.png")} width="1536" height="1024"/>
            // Keep carton closures plain; no insignia or lettering is part of this scene.
            <g fill="#23455a" data-plain-carton-tops="true">
                <path d="M986 172H1002V180H986Z M1010 168H1027V177H1010Z M1035 162H1050V172H1035Z M1062 166H1079V178H1062Z"/>
            </g>
        </svg>
    })
}
