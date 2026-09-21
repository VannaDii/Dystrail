use super::{outcome_key, percent, round_key, t};
use crate::components::ui::stat_card::{self, StatCard};
use crate::game::{
    boss::{HearingOutcome, HearingPhase, HearingReport},
    journal::ResourceChange,
};
use yew::prelude::*;

fn tone(before: f64, after: f64) -> &'static str {
    if after < before {
        "harmful"
    } else if after > before {
        "helpful"
    } else {
        "neutral"
    }
}

fn questioning_closed(report: &HearingReport, phase: HearingPhase) -> bool {
    report.outcome != HearingOutcome::Exhausted
        && match phase {
            HearingPhase::CommitteeResult(i) => usize::from(i) + 1 == report.rounds.len(),
            HearingPhase::Closed
            | HearingPhase::VoteRolling
            | HearingPhase::Verdict
            | HearingPhase::Complete => true,
            _ => false,
        }
}

pub(crate) fn rounds(report: &HearingReport, phase: HearingPhase) -> Html {
    html! {<ol class="hearing-rounds">{for report.rounds.iter().take(report.revealed_rounds(phase)).enumerate().map(|(i,r)|{
        let before = report.adjusted_through(i).min(1.0);
        let after = if r.sanity_after <= 0 { 0.0 } else { report.adjusted_through(i+1).min(1.0) };
        html!{<li data-round={(i+1).to_string()}>
            <div class="hearing-round-heading"><span class="hearing-round-number" aria-hidden="true">{i+1}</span><strong>{t(round_key(u8::try_from(i).unwrap_or(2)))}</strong></div>
            <ul class="resource-changes hearing-round-stats">
                {StatCard {key:"hearing.influence".into(), value:percent(f64::from(r.influence)/100.0), subtitle:String::new(), tone:tone(100.0,f64::from(r.influence))}.render()}
                {stat_card::change(&ResourceChange {key:"ux.sanity".into(),before:i64::from(r.sanity_before),after:i64::from(r.sanity_after)}).render()}
                {StatCard {key:"hearing.odds".into(), value:percent(after), subtitle:format!("{} → {}",percent(before),percent(after)), tone:tone(before,after)}.render()}
            </ul>
        </li>}
    })}
    if questioning_closed(report,phase) {<li class="hearing-questioning-closed"><strong>{t("no_questions_title")}</strong><p>{t("no_questions")}</p></li>}
    </ol>}
}

pub(crate) fn resolution(report: &HearingReport) -> Html {
    html! {<div class="hearing-resolution">
        if let Some(adjusted)=report.adjusted_chance {
            <p class="hearing-equation"><span>{percent(report.base_chance)}<small>{t("starting_odds")}</small></span>if !report.policy_guarantee {<b class="hearing-operator" aria-hidden="true">{"×"}</b><span>{crate::i18n::fmt_number(report.average_through(report.rounds.len()).round()/100.0)}<small>{t("average")}</small></span><b class="hearing-operator" aria-hidden="true">{"≈"}</b>}<span>{percent(adjusted)}<small>{t("final_odds")}</small></span></p>
            if report.policy_guarantee {<p>{t("guarantee")}</p>}
        } else {<p>{t("exhausted_body")}</p>}
    </div>}
}

pub(crate) fn scorecard(report: &HearingReport) -> Html {
    html! {<section class="hearing-scorecard" aria-labelledby="hearing-record-title">
        <h2 id="hearing-record-title">{t("record")}</h2><p><strong>{t(outcome_key(report.outcome))}</strong></p>
        {rounds(report,HearingPhase::Complete)}{resolution(report)}
    </section>}
}
