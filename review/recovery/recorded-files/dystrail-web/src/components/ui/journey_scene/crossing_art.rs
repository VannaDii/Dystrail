//! Source settings and outcome positions; crew and simulation are never baked in.
use super::{Props,SceneStage};
use crate::game::state::CrossingOutcomeTelemetry;
use yew::prelude::*;
pub fn cell(unit:&str)->Option<(u16,u16)> {
    match unit {"CROSS-01-A"=>Some((0,0)),"CROSS-02C-A"=>Some((768,0)),"CROSS-02D-A"=>Some((0,512)),"CROSS-03-A"=>Some((768,512)),_=>None}
}
pub fn context(stage:&SceneStage)->Option<(&str,CrossingOutcomeTelemetry)> {
    if let SceneStage::Crossing{unit,outcome}=stage {cell(unit)?;Some((unit,*outcome))}else{None}
}
pub fn render(p:&Props)->Option<Html> {
    let(unit,outcome)=context(&p.stage)?;let(x,y)=cell(unit)?;
    let result=match outcome {CrossingOutcomeTelemetry::Passed=>"passed",CrossingOutcomeTelemetry::Detoured=>"detoured",CrossingOutcomeTelemetry::Failed=>"failed"};
    // The gate line is between approach and far-side positions. Detours use the
    // visible branch; failure never invents a crash or a dead traveler.
    Some(html!{<div class="crossing-composition" data-crossing-unit={unit.to_owned()} data-crossing-outcome={result}>
        <svg class="scene-background" viewBox={format!("{x} {y} 768 512")} preserveAspectRatio="xMidYMid meet"><image href={crate::paths::asset_path("static/img/scenes-v2/crossing-settings-a-20260915.png")} width="1536" height="1024" /></svg>
        <super::van::CrewVan party={Some(p.party.clone().unwrap_or_default())} />
        <svg class="crossing-barrier" viewBox="0 0 768 512" aria-hidden="true" shape-rendering="crispEdges">
            <path d="M432 332V413H444V332Z" fill="#323c47"/>
            if outcome==CrossingOutcomeTelemetry::Passed {<path d="M430 330V255H442V330Z" fill="#f0d886"/>}else{<path d="M438 334L403 400L413 405L449 339Z" fill="#f0d886"/>}
        </svg>
    </div>})
}
