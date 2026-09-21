//! Once-per-activation presentation of engine events; never apply their effects here.
use super::{Phase, state::AppState};
use crate::{
    components::ui::stats_bar::{self, HudPart, StatsBar},
    game::{GameState, exec_orders::ExecOrder, journal::PolicyBulletin},
    i18n,
};
use std::rc::Rc;
use yew::prelude::*;

pub fn pending(gs: &GameState) -> Option<&PolicyBulletin> {
    gs.continuity
        .visual_content
        .policy_bulletins
        .iter()
        .find(|notice| !notice.acknowledged)
}

pub fn is_pending(app: &AppState) -> bool {
    app.session.as_ref().is_some_and(|s| pending(s.state()).is_some())
        && !matches!(*app.phase, Phase::Boot | Phase::Persona | Phase::Crew | Phase::Outfitting | Phase::Menu)
}

/// Inspect only events appended by this committed action. An old saved active
/// order is not a new activation. Log-position IDs distinguish repeat activations.
pub fn capture(before: &GameState, after: &mut GameState) {
    let starts: Vec<_> = after.logs.iter().enumerate().skip(before.logs.len())
        .filter_map(|(index, line)| {
            let key = line.strip_prefix("exec.start.")?;
            ExecOrder::ALL.iter().find(|order| order.key() == key)
                .map(|order| (format!("policy/{index}"), *order))
        }).collect();
    for (id, order) in starts {
        if after.continuity.visual_content.policy_bulletins.iter().any(|n| n.id == id) {
            continue;
        }
        let unit = super::visual_content::select(after, super::workshop::order(order), &id);
        after.continuity.visual_content.policy_bulletins.push(PolicyBulletin {
            id, order, unit,
            received_day: after.day,
            received_minute: after.continuity.clock_minutes,
            acknowledged: false,
        });
    }
}

pub fn acknowledge(gs: &mut GameState, id: &str) {
    if let Some(notice) = gs.continuity.visual_content.policy_bulletins.iter_mut().find(|n| n.id == id) {
        notice.acknowledged = true;
    }
}

pub fn active_unit(gs: &GameState, order: ExecOrder) -> String {
    let family = super::workshop::order(order);
    let variant = gs.continuity.visual_content.policy_bulletins.iter().rev()
        .find(|notice| notice.order == order && still_active(gs,notice))
        .map_or("A", |notice| super::visual_content::variant(family,&notice.unit));
    format!("{family}-{variant}")
}

pub fn render(app: &AppState) -> Html {
    let Some(gs) = app.session.as_ref().map(|s| s.state()) else { return Html::default(); };
    let Some(notice) = pending(gs).cloned() else { return Html::default(); };
    let on_continue = {
        let app = app.clone();
        let id = notice.id.clone();
        Callback::from(move |()| {
            let Some(mut session) = (*app.session).clone() else { return; };
            session.with_state_mut(|gs| acknowledge(gs, &id));
            let next = super::aftermath::next_phase(session.state());
            let clear = pending(session.state()).is_none();
            if clear && app.aftermath.is_none() {
                app.phase.set(next);
                app.map_automatic.set(false);
                app.travel_running.set(next == Phase::Travel && session.state().breakdown.is_none());
            }
            app.session.set(Some(session));
        })
    };
    let key = notice.id.clone();
    html! {<Bulletin {key} state={Rc::new(gs.clone())} {notice} {on_continue} />}
}

#[derive(Properties)]
struct Props {
    state: Rc<GameState>,
    notice: PolicyBulletin,
    on_continue: Callback<()>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
            && self.notice == other.notice
            && self.on_continue == other.on_continue
    }
}

#[function_component(Bulletin)]
fn bulletin(p: &Props) -> Html {
    i18n::use_language();
    let heading = use_node_ref();
    {
        let heading = heading.clone();
        use_effect_with((), move |()| {
            if let Some(el) = heading.cast::<web_sys::HtmlElement>() { let _ = el.focus(); }
        });
    }
    let gs = &p.state;
    let notice = &p.notice;
    let family = super::workshop::order(notice.order);
    let title = super::workshop::text(family, "title");
    let cell = ExecOrder::ALL.iter().position(|order| *order == notice.order).unwrap_or(0);
    let x = (cell % 2) * 768;
    let y = (cell / 2) as f64 * (1024.0 / 3.0);
    let clock = format!("{:02}:{:02}", notice.received_minute / 60, notice.received_minute % 60);
    let received = i18n::tr("policy_bulletin.received", Some(&std::collections::BTreeMap::from([
        ("day", i18n::fmt_number(f64::from(notice.received_day)).as_str()),
        ("time", clock.as_str()),
    ])));
    let active = gs.current_order == Some(notice.order);
    let cfg = crate::game::WeatherConfig::default_config();
    let count = if notice.order == ExecOrder::WarDeptReorg {2} else {1};
    let on_continue = {let cb=p.on_continue.clone();Callback::from(move |_|cb.emit(()))};
    html! {<section class="policy-bulletin" data-bulletin={notice.id.clone()} data-unit={notice.unit.clone()} aria-labelledby="bulletin-title">
        <StatsBar part={HudPart::Resources} stats={gs.stats.clone()} receipts={gs.receipts.len()} day={gs.day} region={gs.region} />
        <StatsBar part={HudPart::Conditions} stats={gs.stats.clone()} receipts={gs.receipts.len()} day={gs.day} region={gs.region}
            clock_hour={u8::try_from(gs.continuity.clock_minutes/60).unwrap_or(8)} clock_minute={u8::try_from(gs.continuity.clock_minutes%60).unwrap_or(0)}
            exec_order={gs.current_order} policy_readout={gs.current_order.map(|order|stats_bar::policy::readout(gs,order))}
            weather={Some(super::phase::build_weather_badge(gs,&cfg))} weather_readout={Some(stats_bar::weather::readout(gs,&cfg))}
            trip_destination={crate::components::ui::leg_summary::render_hud_destination(gs)} />
        <header class="bulletin-heading"><p class="eyebrow">{i18n::t("policy_bulletin.label")}</p><h1 id="bulletin-title" ref={heading} tabindex="-1">{title}</h1><p>{received}</p></header>
        <figure class="bulletin-illustration">
            <svg viewBox={format!("{x} {y} 768 {}",1024.0/3.0)} preserveAspectRatio="xMidYMid meet" aria-hidden="true" data-atlas="policy-bulletins-a" data-cell={cell.to_string()}>
                <image href={crate::paths::asset_path("static/img/scenes-v2/policy-bulletins-a.png")} width="1536" height="1024"/>
            </svg>
            <figcaption>{for (0..count).map(|i|html!{<span>{i18n::t(&format!("policy_art.{family}.{i}"))}</span>})}</figcaption>
        </figure>
        <div class="bulletin-body"><p class="bulletin-copy">{super::workshop::text(family,"activation")}</p>
            {crate::components::ui::satire_context::hook(family)}
            if active { {stats_bar::policy::details(&stats_bar::policy::readout(gs,notice.order))} }
            else {<p class="bulletin-expired">{i18n::t("policy_bulletin.expired")}{" "}{super::workshop::text(family,"expiration")}</p>}
            <div class="bulletin-actions"><button id="bulletin-continue" class="retro-btn-primary" onclick={on_continue}>{i18n::t("ux.continue")}</button></div>
        </div>
    </section>}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_activations_are_queued_once_without_replaying_effects() {
        let before = GameState::default();
        let mut after = before.clone();
        after.current_order = Some(ExecOrder::TariffTsunami);
        after.logs.push("exec.start.tariff_tsunami".into());
        after.stats.supplies -= 1;
        let original = serde_json::to_value(&after).unwrap();
        capture(&before, &mut after);
        capture(&before, &mut after);
        assert_eq!(after.continuity.visual_content.policy_bulletins.len(),1);
        let id = pending(&after).unwrap().id.clone();
        let mut restored: GameState = serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
        assert_eq!(pending(&restored).unwrap().id,id);
        acknowledge(&mut restored,&id);
        acknowledge(&mut restored,&id);
        assert!(pending(&restored).is_none());
        let mut actual = serde_json::to_value(&restored).unwrap();
        actual["visual_content"] = original["visual_content"].clone();
        assert_eq!(actual,original);
        // Continued activity under the same policy is not another bulletin.
        let before = restored.clone();
        capture(&before,&mut restored);
        assert!(pending(&restored).is_none());
        // A later activation of the same order has a distinct identity.
        restored.logs.push("exec.start.tariff_tsunami".into());
        capture(&before,&mut restored);
        assert_ne!(pending(&restored).unwrap().id,id);
    }

    #[test]
    fn legacy_active_orders_and_completed_bulk_events_recover_safely() {
        let before = GameState {
            current_order: Some(ExecOrder::Shutdown),
            ..GameState::default()
        };
        let mut after = before.clone();
        capture(&before,&mut after);
        assert!(pending(&after).is_none());
        after.logs.extend(["exec.start.book_panic".into(),"exec.end.book_panic".into()]);
        after.current_order = None;
        after.continuity.crew_care.pending = Some("journalist".into());
        capture(&before,&mut after);
        assert_eq!(pending(&after).unwrap().order,ExecOrder::BookPanic);
        assert!(super::super::aftermath::next_phase(&after) == Phase::CrewCare);
    }
}
