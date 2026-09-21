//! Player-paced presentation of a hearing that has already been committed to the save.
mod scene;
pub(crate) mod summary;
mod timing;

use crate::app::view::handlers::HearingCommand;
use crate::components::ui::stats_bar::WeatherBadge;
use crate::game::{
    BossConfig, GameState,
    boss::{HearingOutcome, HearingPhase},
};
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct BossPageProps {
    pub state: GameState,
    pub config: BossConfig,
    pub weather: WeatherBadge,
    pub on_begin: Callback<()>,
    pub on_hearing: Callback<HearingCommand>,
    pub on_fast: Callback<bool>,
    #[prop_or_default]
    pub fast: bool,
    #[prop_or_default]
    pub paused: bool,
    #[prop_or_default]
    pub on_camp: Callback<()>,
}
impl PartialEq for BossPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.state.boss.hearing == other.state.boss.hearing
            && self.state.boss.presentation == other.state.boss.presentation
            && self.state.stats == other.state.stats
            && self.state.day == other.state.day
            && self.state.continuity.clock_minutes == other.state.continuity.clock_minutes
            && self.state.party == other.state.party
            && self.config == other.config
            && self.fast == other.fast
            && self.paused == other.paused
            && self.on_hearing == other.on_hearing
            && self.on_begin == other.on_begin
            && self.on_fast == other.on_fast
            && self.on_camp == other.on_camp
    }
}

pub(super) fn t(key: &str) -> String {
    crate::i18n::t(&format!("hearing.{key}"))
}
pub(super) fn text(key: &str, args: &[(&str, String)]) -> String {
    crate::i18n::tr(
        &format!("hearing.{key}"),
        Some(&args.iter().map(|(k, v)| (*k, v.as_str())).collect()),
    )
}
pub(super) fn percent(value: f64) -> String {
    // Never round a chance below 100% up to a displayed guarantee.
    format!(
        "{}%",
        crate::i18n::fmt_number((value * 10000.0 + 1e-7).floor() / 100.0)
    )
}
pub(super) const fn round_key(index: u8) -> &'static str {
    match index {
        0 => "opening",
        1 => "followup",
        _ => "challenge",
    }
}
pub(super) const fn outcome_key(outcome: HearingOutcome) -> &'static str {
    match outcome {
        HearingOutcome::Passed => "passed",
        HearingOutcome::Failed => "failed",
        HearingOutcome::Secured => "secured",
        HearingOutcome::Exhausted => "exhausted",
    }
}

#[function_component(BossPage)]
pub fn boss_page(p: &BossPageProps) -> Html {
    crate::i18n::use_language();
    let phase = p.state.boss.presentation;
    let report = p.state.boss.hearing.as_ref();
    timing::use_presentation(p);
    let forecast = crate::game::boss::hearing_forecast(&p.state, &p.config);
    let count = report.map_or(0, |r| r.revealed_rounds(phase));
    let sanity = report.map_or(p.state.stats.sanity, |r| {
        r.rounds
            .get(count.wrapping_sub(1))
            .map_or(r.starting_stats.sanity, |round| round.sanity_after)
    });
    let base = report.map_or(forecast.base_chance, |r| r.base_chance);
    let title = match phase {
        HearingPhase::Arrival => t("arrival"),
        HearingPhase::Preparation => t("preparation"),
        HearingPhase::RoundRolling(i) | HearingPhase::RoundResult(i) => t(round_key(i)),
        HearingPhase::Committee(_) | HearingPhase::CommitteeResult(_) => t("committee"),
        HearingPhase::Closed => t("closed"),
        HearingPhase::VoteRolling => t("vote"),
        _ => report.map_or_else(|| t("closed"), |r| t(outcome_key(r.outcome))),
    };
    let advance = {
        let on = p.on_hearing.clone();
        Callback::from(move |event: MouseEvent| {
            if event.detail() <= 1 {
                on.emit(HearingCommand::Advance(phase));
            }
        })
    };
    let verdict = matches!(phase, HearingPhase::Verdict | HearingPhase::Complete);
    let animated = timing::duration(phase, false, false).is_some();
    let narration = match phase {
        HearingPhase::Arrival => t("arrival_body"),
        HearingPhase::Preparation => t("preparation_body"),
        HearingPhase::RoundRolling(i) => t(&format!("{}_question", round_key(i))),
        HearingPhase::RoundResult(i) => report
            .and_then(|r| r.rounds.get(usize::from(i)))
            .map_or_else(String::new, |r| {
                let reaction = if r.influence >= 110 {
                    "positive"
                } else if r.influence <= 90 {
                    "negative"
                } else {
                    "neutral"
                };
                t(&format!("{}_{reaction}", round_key(i)))
            }),
        HearingPhase::Committee(_) => t("committee_wait"),
        HearingPhase::CommitteeResult(i) => t(if count < report.map_or(0, |r| r.rounds.len()) {
            "called"
        } else {
            let _ = i;
            "no_questions"
        }),
        HearingPhase::Closed => t("closed_body"),
        HearingPhase::VoteRolling => t("vote_wait"),
        _ => report.map_or_else(String::new, |r| {
            t(&format!("{}_body", outcome_key(r.outcome)))
        }),
    };
    html! {<section class={classes!("hearing",p.fast.then_some("hearing-fast"))} data-hearing-phase={format!("{phase:?}")} data-revealed-rounds={count.to_string()} aria-labelledby="hearing-title">
        <div class="hearing-hud">
            <div><span>{t("starting_odds")}</span><strong>{percent(base)}</strong></div>
            <div class={classes!((sanity<=2).then_some("hearing-low-sanity"))}><span>{crate::i18n::t("ux.sanity")}</span><strong data-hearing-sanity="true">{sanity}</strong></div>
            <div class="hearing-round-label">{match phase {
                HearingPhase::RoundRolling(i) | HearingPhase::RoundResult(i) | HearingPhase::Committee(i) | HearingPhase::CommitteeResult(i) => text("round_label",&[("round",(i+1).to_string()),("max",p.config.rounds.clamp(1,3).to_string())]),
                _ => t("location"),
            }}</div>
            <button class="fast-mode-toggle" role="switch" aria-label={crate::i18n::t("journey.fast_mode")} aria-checked={p.fast.to_string()} onclick={{let on=p.on_fast.clone();let fast=p.fast;Callback::from(move |_|on.emit(!fast))}}><span class="switch-track" aria-hidden="true"><span /></span>{crate::i18n::t("journey.fast")}</button>
        </div>
        {scene::render(p, &title)}
        <div class="hearing-content">
            <div class="hearing-narrative" aria-live="polite" aria-atomic="true">
                <p class="eyebrow">{t("location")}</p><h1 id="hearing-title" tabindex="-1">{&title}</h1>
                <p class="hearing-dialogue">{&narration}</p>
            </div>
            if phase == HearingPhase::Preparation {
                <div class="hearing-brief">
                    if forecast.preparation_supplies > 0 || forecast.preparation_cents > 0 {
                        <p class="hearing-preparation-cost">{t(if forecast.preparation_supplies > 0 {"preparation_supplies"} else {"preparation_cash"})}</p>
                    }
                    <p class={classes!((forecast.survival_chance < 1.0).then_some("hearing-warning"))}>{t(if forecast.survival_chance < 1.0 {"rest_warning"} else {"preparation_ready"})}</p>
                    if forecast.policy_guarantee {<p>{t("guarantee")}</p>}
                </div>
            } else if let Some(report) = report {
                if count > 0 { {summary::rounds(report, phase)} }
                if (verdict || phase == HearingPhase::Closed) && report.adjusted_chance.is_some() { {summary::resolution(report)} }
            }
            <div class="hearing-controls">
                if phase == HearingPhase::Preparation {
                    <button onclick={{let on=p.on_camp.clone();Callback::from(move |_|on.emit(()))}}>{crate::i18n::t("journey.review_before_vote")}</button>
                    <button id="hearing-next" class="retro-btn-primary" onclick={{let on=p.on_begin.clone();Callback::from(move |event:MouseEvent|{if event.detail()<=1 {on.emit(());}})}}>{t("begin")}</button>
                } else if !animated || phase == HearingPhase::Arrival {
                    <button id="hearing-next" class="retro-btn-primary" onclick={advance}>{if verdict {t("scorecard")} else if phase == HearingPhase::Closed {t("reveal")} else if phase == HearingPhase::Arrival {t("enter")} else {crate::i18n::t("ui.continue")}}</button>
                } else {<p class="hearing-wait" role="status"><span aria-hidden="true" class="hearing-dots">{"• • •"}</span>{t("in_progress")}</p>}
                if report.is_some() && !verdict && phase != HearingPhase::Closed {<button class="hearing-skip" onclick={{let on=p.on_hearing.clone();Callback::from(move |event:MouseEvent|{if event.detail()<=1 {on.emit(HearingCommand::Skip(phase));}})}}>{t("skip")}</button>}
            </div>
        </div>
    </section>}
}
