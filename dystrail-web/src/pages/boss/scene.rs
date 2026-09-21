use super::{BossPageProps, t};
use crate::components::ui::character_portrait::{self, Expression};
use crate::components::ui::journey_scene::{JourneyScene, SceneStage};
use crate::game::{
    boss::{HearingOutcome, HearingPhase},
    party::MemberStatus,
};
use yew::prelude::*;

pub(super) fn render(p: &BossPageProps, title: &str) -> Html {
    let phase = p.state.boss.presentation;
    let arrival = phase == HearingPhase::Arrival;
    let preparation = phase == HearingPhase::Preparation;
    let report = p.state.boss.hearing.as_ref();
    let verdict = matches!(phase, HearingPhase::Verdict | HearingPhase::Complete);
    let outcome = report.filter(|_| verdict).map(|r| r.outcome);
    let expression = outcome.map_or(Expression::Standard, Expression::hearing);
    let active: Vec<_> = p
        .state
        .party
        .members
        .iter()
        .filter(|m| m.status == MemberStatus::Active)
        .collect();
    let speaker = crate::components::ui::journey_scene::composition::subject(
        &p.state.party,
        p.state.persona_id.as_deref(),
        p.state.day,
        "hearing",
    );
    let committee = matches!(
        phase,
        HearingPhase::Committee(_) | HearingPhase::CommitteeResult(_) | HearingPhase::VoteRolling
    );
    let cell = if verdict {
        match outcome {
            Some(HearingOutcome::Passed | HearingOutcome::Secured) => 4,
            _ => 5,
        }
    } else if phase == HearingPhase::VoteRolling {
        3
    } else if committee {
        1
    } else if let HearingPhase::RoundResult(i) = phase {
        report
            .and_then(|r| r.rounds.get(usize::from(i)))
            .map_or(0, |r| {
                if r.influence >= 110 {
                    2
                } else {
                    i32::from(r.influence <= 90)
                }
            })
    } else {
        0
    };
    let closed = verdict || matches!(phase, HearingPhase::Closed | HearingPhase::VoteRolling);
    let day = report.filter(|_| !closed).map_or(p.state.day, |r| r.day);
    let minute = report
        .filter(|_| !closed)
        .map_or(p.state.continuity.clock_minutes, |r| r.minute);
    let hour = u8::try_from(minute / 60).unwrap_or(12);
    html! {<div class={classes!("hearing-stage",arrival.then_some("hearing-arrival"),preparation.then_some("hearing-preparation"),committee.then_some("hearing-committee"),verdict.then_some("hearing-verdict"),outcome.map(|o|format!("hearing-ending-{}",super::outcome_key(o))))}>
        <JourneyScene stage={if arrival {SceneStage::Ending(true)} else {SceneStage::Boss}} show_cast={false} {day} {hour} party={Some(p.state.party.clone())}>
            if arrival { {crate::components::ui::journey_scene::parked::render(Some(&p.state.party))} }
            else {
                <div class="hearing-chair"><figure class="character-portrait"><span class="character-art" aria-hidden="true"><svg viewBox={format!("{} {} 512 512",(cell%3)*512,(cell/3)*512)} focusable="false"><image href={crate::paths::asset_path("static/img/journey/hearing-officials-v1.png")} width="1536" height="1024"/></svg></span><figcaption>{t(if cell>=3 {"clerk"} else {"chair"})}</figcaption></figure></div>
                if let Some(speaker)=speaker {<div class="hearing-speaker">{character_portrait::framed(&speaker.persona,&speaker.name,expression)}</div>}
                if matches!(phase,HearingPhase::RoundRolling(_)|HearingPhase::VoteRolling|HearingPhase::Committee(_)) {<div class="hearing-paper" aria-hidden="true"><span/><span/><span/></div>}
            }
            <figcaption class="hearing-scene-caption"><span>{title}</span><crate::components::ui::game_clock::GameClock {day} {hour} minute={u8::try_from(minute%60).unwrap_or(0)} moving={false}/></figcaption>
        </JourneyScene>
        <div class="hearing-crew" role="group" aria-label={p.state.party.name.clone()}>{for active.iter().filter(|m|arrival || speaker.is_none_or(|s|s.persona!=m.persona)).map(|m|character_portrait::framed(&m.persona,&m.name,expression))}</div>
    </div>}
}
