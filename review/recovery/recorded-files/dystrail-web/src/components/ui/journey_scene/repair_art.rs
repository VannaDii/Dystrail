//! Shared component close-ups and current crew for every repair variant.
use super::{Props, SceneStage};
use yew::prelude::*;

pub fn context(stage: &SceneStage) -> Option<(&str, usize, usize, Option<usize>)> {
    let SceneStage::Repair { unit, choice } = stage else { return None; };
    let (family, variant) = unit.rsplit_once('-')?;
    let row = match family {
        "REPAIR-TIRE" => 0, "REPAIR-BATTERY" => 1,
        "REPAIR-ALTERNATOR" => 2, "REPAIR-FUELPUMP" => 3,
        _ => return None,
    };
    let variant = match variant { "A"=>0,"B"=>1,"C"=>2,_=>return None };
    if choice.is_some_and(|c|c>3) { return None; }
    Some((unit, row, variant, *choice))
}

fn tile(path: &str, view: String, width: u32, height: u32, class: &str) -> Html {
    html!{<svg class={class.to_owned()} viewBox={view} preserveAspectRatio="xMidYMid meet" overflow="hidden" aria-hidden="true">
        <image href={crate::paths::asset_path(path)} width={width.to_string()} height={height.to_string()} preserveAspectRatio="none"/>
    </svg>}
}

pub fn render(p: &Props) -> Option<Html> {
    let (unit, row, variant, choice) = context(&p.stage)?;
    let road = p.road_asset.as_deref().unwrap_or_else(||super::road_asset(p.region.unwrap_or(crate::game::Region::Heartland),p.day));
    let repaired=choice.is_some();
    let fault=tile("static/img/scenes-v2/repair-components-20260914.png",format!("{} {} 768 256",usize::from(repaired)*768,row*256),1536,1024,"repair-component");
    // Prop sheet rows follow alternator, battery, pump, tire; fault sheet follows
    // tire, battery, alternator, pump. Keep this explicit instead of relying on enum order.
    let props_row=[3,1,0,2][row];
    let props=tile("static/img/scenes-v2/repair-props-20260914.png",format!("{} {} 512 256",variant*512,props_row*256),1536,1024,"repair-props");
    let subject=p.party.as_ref().and_then(|party|super::composition::subject(party,p.subject.as_deref(),0,unit));
    Some(html!{<div class="repair-composition" data-repair-unit={unit.to_owned()} data-repair-state={if repaired{"repaired"}else{"failed"}} data-repair-choice={choice.map(|c|c.to_string())}>
        <img class="scene-background" src={crate::paths::asset_path(&format!("static/img/journey/{road}.png"))} alt="" decoding="sync" width="1536" height="1024"/>
        <div class="repair-van"><super::van::CrewVan party={p.party.clone()} empty={true}/></div>
        {fault}
        if let Some(member)=subject {
            <div class="repair-inspector" data-member={member.persona.clone()}>
                {inspection(&member.persona)}
                {props}
            </div>
        }
    </div>})
}

fn inspection(persona:&str)->Html {
    let cell=match persona {"organizer"=>1,"whistleblower"=>2,"lobbyist"=>3,"staffer"=>4,"satirist"=>5,_=>0};
    tile("static/img/cast-v2/inspection-20260914.png",format!("{} {} 512 512",(cell%3)*512,(cell/3)*512),1536,1024,"repair-inspection-pose")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn component_mapping_distinguishes_offers_from_committed_repairs() {
        for (row,part) in ["TIRE","BATTERY","ALTERNATOR","FUELPUMP"].iter().enumerate(){
            for variant in ["A","B","C"]{
                for choice in [None,Some(0),Some(1),Some(2),Some(3)]{
                    let stage=SceneStage::Repair {unit:format!("REPAIR-{part}-{variant}"),choice};
                    let (_,actual,_,outcome)=context(&stage).unwrap(); assert_eq!(actual,row);assert_eq!(outcome,choice);
                }
            }
        }
        for unit in ["REPAIR-TIRE-Z","ENC-C01-A","REPAIR-BRAKE-A"]{
            assert!(context(&SceneStage::Repair{unit:unit.into(),choice:None}).is_none());
        }
        assert!(context(&SceneStage::Repair{unit:"REPAIR-TIRE-A".into(),choice:Some(4)}).is_none());
    }
}
