//! Authored care settings. The saved incident chooses its own scene, never a random crew member.
use super::{CareOutcome, Props, SceneStage};
use yew::prelude::*;

#[derive(Clone, Copy)]
enum Pose { Mild, Severe, Recovered, Walking, Slumped, SoreBack }
impl Pose {
    const fn name(self) -> &'static str {
        match self { Self::Mild=>"mild",Self::Severe=>"severe",Self::Recovered=>"recovered",Self::Walking=>"walking",Self::Slumped=>"slumped",Self::SoreBack=>"sore-back" }
    }
}

fn character(persona: &str, pose: Pose, in_van: bool) -> Html {
    let col = match persona { "journalist"=>0,"organizer"=>1,"whistleblower"=>2,"lobbyist"=>3,"staffer"=>4,"satirist"=>5,_=>return Html::default() };
    let actions = matches!(pose,Pose::Walking|Pose::Slumped|Pose::SoreBack);
    let row = match pose { Pose::Mild|Pose::Walking=>0,Pose::Severe|Pose::Slumped=>1,Pose::Recovered|Pose::SoreBack=>2 };
    let xs=[0,220,433,635,843,1045,1254];
    let ys=if actions {[0,478,842,1254]}else{[0,478,869,1254]};
    let height=if in_van {260}else{ys[row+1]-ys[row]};
    let stem=if actions {"care-actions-20260915"}else{"care-seated-20260915"};
    html! {<svg class="care-pose" viewBox={format!("{} {} {} {height}",xs[col],ys[row],xs[col+1]-xs[col])} overflow="hidden" aria-hidden="true">
        <defs>
            <filter id="care-mask-edge" color-interpolation-filters="sRGB">
                <feComponentTransfer><feFuncR type="discrete" tableValues="0 1"/><feFuncG type="discrete" tableValues="0 1"/><feFuncB type="discrete" tableValues="0 1"/></feComponentTransfer>
                <feMorphology operator="dilate" radius="2"/><feMorphology operator="erode" radius="3"/>
            </filter>
            <mask id="care-character-mask" maskUnits="userSpaceOnUse" x="0" y="0" width="1254" height="1254" style="mask-type:luminance">
                <image href={crate::paths::asset_path(&format!("static/img/cast-v2/{stem}-mask.png"))} width="1254" height="1254" preserveAspectRatio="none" filter="url(#care-mask-edge)"/>
            </mask>
        </defs>
        <image href={crate::paths::asset_path(&format!("static/img/cast-v2/{stem}.png"))} width="1254" height="1254" preserveAspectRatio="none" mask="url(#care-character-mask)"/>
    </svg>}
}

pub fn context(stage: &SceneStage) -> Option<(&str, u8, &str)> {
    let SceneStage::CareIncident { unit, outcome: CareOutcome::Offered | CareOutcome::Deferred, .. } = stage else {return None;};
    let (family,variant)=unit.rsplit_once('-')?;
    if !matches!(variant,"A"|"B"|"C") {return None;}
    let cell=match family {
        "CARE-01"=>0,"CARE-02"=>1,"CARE-03"=>2,"CARE-04"=>3,
        "CARE-05"=>4,"CARE-06"=>5,"CARE-07"=>6,"CARE-08"=>7,_=>return None,
    };
    Some((unit,cell,variant))
}

pub fn indoors(stage: &SceneStage) -> bool {
    context(stage).is_some_and(|(_,cell,v)| matches!((cell,v),(2,"A"|"B"|"C")|(3,"B")|(5,"B"|"C")|(4|7,"C")))
}

pub fn render(p: &Props) -> Option<Html> {
    let (unit,cell,variant)=context(&p.stage)?;
    let x=u32::from(cell%2)*768;
    let y=u32::from(cell/2)*512;
    let path=format!("static/img/scenes-v2/care-settings-{}-20260914.png",variant.to_ascii_lowercase());
    let sign=crate::i18n::t(&format!("visual_copy.{unit}.sign"));
    Some(html! {<>
        <svg class="scene-background care-setting" data-care-setting={unit.to_owned()} viewBox={format!("{x} {y} 768 512")} preserveAspectRatio="xMidYMid meet" overflow="hidden" aria-hidden="true">
            <image href={crate::paths::asset_path(&path)} width="1536" height="2048" preserveAspectRatio="none"/>
        </svg>
        if !sign.is_empty() {<div class="care-sign" data-care-sign={unit.to_owned()} dir="auto">{sign}</div>}
    </>})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_valid_pending_care_uses_incident_backdrops() {
        for n in 1..=8 {for v in ["A","B","C"] {
            let mut stage=SceneStage::CareIncident{unit:format!("CARE-{n:02}-{v}"),persona:"organizer".into(),strain:1,outcome:CareOutcome::Offered};
            assert_eq!(context(&stage).unwrap().1,n-1);
            if let SceneStage::CareIncident{outcome,..}=&mut stage {*outcome=CareOutcome::Sheltered;}
            assert!(context(&stage).is_none());
        }}
        for unit in ["CARE-00-A","CARE-09-B","CARE-01-Z","ACT-REST-A"] {
            assert!(context(&SceneStage::CareIncident{unit:unit.into(),persona:"organizer".into(),strain:1,outcome:CareOutcome::Offered}).is_none());
        }
    }
}
