//! Presentation receipts for committed engine activations; no simulation effects.
use crate::game::{GameState, exec_orders::ExecOrder, journal::PolicyBulletin};

pub fn family(order: ExecOrder) -> &'static str {
    match order {
        ExecOrder::Shutdown => "ORDER-SHUTDOWN",
        ExecOrder::TravelBanLite => "ORDER-MILITARIZE",
        ExecOrder::BookPanic => "ORDER-GAG",
        ExecOrder::TariffTsunami => "ORDER-TARIFFS",
        ExecOrder::DoEEliminated => "ORDER-TAXCUTS",
        ExecOrder::WarDeptReorg => "ORDER-DEREGULATE",
    }
}

pub fn pending(gs: &GameState) -> Option<&PolicyBulletin> {
    gs.continuity
        .visual_content
        .policy_bulletins
        .iter()
        .find(|n| !n.acknowledged)
}

/// Log positions identify repeated activations. Inspect only new committed logs,
/// so loading an old active order cannot manufacture a fresh announcement.
pub fn capture(before: &GameState, after: &mut GameState) {
    if after.continuity.visual_content.edition != super::visual_content::EDITION {
        return;
    }
    let starts: Vec<_> = after
        .logs
        .iter()
        .enumerate()
        .skip(before.logs.len())
        .filter_map(|(index, line)| {
            let key = line.strip_prefix("exec.start.")?;
            ExecOrder::ALL
                .iter()
                .find(|order| order.key() == key)
                .map(|order| (index, *order))
        })
        .collect();
    for (index, order) in starts {
        let id = format!("policy/{index}");
        if after
            .continuity
            .visual_content
            .policy_bulletins
            .iter()
            .any(|n| n.id == id)
        {
            continue;
        }
        let hash = family(order)
            .bytes()
            .chain(id.bytes())
            .fold(after.seed ^ 0xcbf2_9ce4_8422_2325, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
            });
        let unit = format!("{}-{}", family(order), ["A", "B", "C"][(hash % 3) as usize]);
        after
            .continuity
            .visual_content
            .policy_bulletins
            .push(PolicyBulletin {
                id,
                order,
                unit,
                received_day: after.day,
                received_minute: after.continuity.clock_minutes,
                acknowledged: false,
            });
    }
}

pub fn acknowledge(gs: &mut GameState, id: &str) {
    if let Some(n) = gs
        .continuity
        .visual_content
        .policy_bulletins
        .iter_mut()
        .find(|n| n.id == id)
    {
        n.acknowledged = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_and_acknowledge_preserve_simulation_and_survive_reload() {
        let mut before = GameState::default();
        before.seed = u64::MAX;
        before.continuity.visual_content.edition = 1;
        let mut after = before.clone();
        after.current_order = Some(ExecOrder::TariffTsunami);
        after.logs.push("exec.start.tariff_tsunami".into());
        after.stats.supplies -= 1;
        let original = serde_json::to_value(&after).unwrap();
        capture(&before, &mut after);
        capture(&before, &mut after);
        assert_eq!(after.continuity.visual_content.policy_bulletins.len(), 1);
        let id = pending(&after).unwrap().id.clone();
        let mut restored: GameState =
            serde_json::from_str(&serde_json::to_string(&after).unwrap()).unwrap();
        assert_eq!(pending(&restored), pending(&after));
        acknowledge(&mut restored, &id);
        acknowledge(&mut restored, &id);
        assert!(pending(&restored).is_none());
        let mut actual = serde_json::to_value(&restored).unwrap();
        actual["visual_content"] = original["visual_content"].clone();
        assert_eq!(actual, original);
        let before = restored.clone();
        capture(&before, &mut restored);
        assert!(pending(&restored).is_none());
        restored.logs.push("exec.start.tariff_tsunami".into());
        capture(&before, &mut restored);
        assert_ne!(pending(&restored).unwrap().id, id);
    }
    #[test]
    fn all_policies_have_stable_variants_and_old_orders_are_not_reannounced() {
        let mut before = GameState::default();
        before.continuity.visual_content.edition = 1;
        before.current_order = Some(ExecOrder::Shutdown);
        let mut after = before.clone();
        capture(&before, &mut after);
        assert!(pending(&after).is_none());
        for order in ExecOrder::ALL {
            after.logs.push(format!("exec.start.{}", order.key()));
            after.logs.push(format!("exec.end.{}", order.key()));
        }
        let mut replay = after.clone();
        capture(&before, &mut after);
        capture(&before, &mut replay);
        assert_eq!(
            after.continuity.visual_content,
            replay.continuity.visual_content
        );
        assert_eq!(after.continuity.visual_content.policy_bulletins.len(), 6);
        for n in &after.continuity.visual_content.policy_bulletins {
            assert!(
                ["A", "B", "C"]
                    .iter()
                    .any(|v| n.unit == format!("{}-{v}", family(n.order)))
            );
        }
        let mut legacy: GameState = serde_json::from_str("{}").unwrap_or_default();
        legacy.logs.push("exec.start.shutdown".into());
        capture(&GameState::default(), &mut legacy);
        assert!(pending(&legacy).is_none());
    }
}

pub fn is_pending(app: &super::state::AppState) -> bool {
    app.session
        .as_ref()
        .is_some_and(|s| pending(s.state()).is_some())
        && !matches!(
            *app.phase,
            super::Phase::Boot
                | super::Phase::Persona
                | super::Phase::Crew
                | super::Phase::Outfitting
                | super::Phase::Menu
                | super::Phase::Boss
                | super::Phase::Result
        )
}

fn current_activation(gs: &GameState, notice: &PolicyBulletin) -> bool {
    if gs.current_order != Some(notice.order) {
        return false;
    }
    let Some(index) = notice
        .id
        .strip_prefix("policy/")
        .and_then(|n| n.parse::<usize>().ok())
    else {
        return false;
    };
    !gs.logs.iter().skip(index.saturating_add(1)).any(|line| {
        line.starts_with("exec.start.") || line == &format!("exec.end.{}", notice.order.key())
    })
}

pub fn render(app: &super::state::AppState) -> yew::Html {
    use yew::prelude::*;
    let Some(gs) = app.session.as_ref().map(|s| s.state()) else {
        return Html::default();
    };
    let Some(notice) = pending(gs).cloned() else {
        return Html::default();
    };
    let id = notice.id.clone();
    let on_continue = {
        let app = app.clone();
        Callback::from(move |()| {
            let Some(mut session) = (*app.session).clone() else {
                return;
            };
            session.with_state_mut(|gs| acknowledge(gs, &id));
            if pending(session.state()).is_none()
                && app.aftermath.is_none()
                && *app.phase != super::Phase::Map
            {
                let next = super::aftermath::next_phase(session.state());
                app.phase.set(next);
                app.travel_running
                    .set(next == super::Phase::Travel && session.state().breakdown.is_none());
            }
            if pending(session.state()).is_none()
                && app.aftermath.is_none()
                && *app.phase == super::Phase::Map
                && *app.map_automatic
            {
                app.travel_running.set(true);
            }
            app.session.set(Some(session));
        })
    };
    let key = notice.id.clone();
    html! {<Bulletin {key} state={std::rc::Rc::new(gs.clone())} {notice} {on_continue}/>}
}

#[derive(yew::Properties)]
struct BulletinProps {
    state: std::rc::Rc<GameState>,
    notice: PolicyBulletin,
    on_continue: yew::Callback<()>,
}
impl PartialEq for BulletinProps {
    fn eq(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.state, &other.state)
            && self.notice == other.notice
            && self.on_continue == other.on_continue
    }
}

#[yew::function_component(Bulletin)]
fn bulletin(p: &BulletinProps) -> yew::Html {
    use crate::{
        components::ui::stats_bar::{self, HudPart, StatsBar},
        i18n,
    };
    use yew::prelude::*;
    i18n::use_language();
    let heading = use_node_ref();
    {
        let heading = heading.clone();
        use_effect_with(p.notice.id.clone(), move |_| {
            use wasm_bindgen::{JsCast, closure::Closure};
            let callback = Closure::wrap(Box::new(move || {
                if let Some(el) = heading.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }) as Box<dyn FnMut()>);
            let window = web_sys::window();
            let timer = window.as_ref().and_then(|w| {
                w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    0,
                )
                .ok()
            });
            move || {
                if let (Some(w), Some(id)) = (window, timer) {
                    w.clear_timeout_with_handle(id);
                }
                drop(callback);
            }
        });
    }
    let gs = &p.state;
    let notice = &p.notice;
    let valid = ["A", "B", "C"]
        .iter()
        .any(|v| notice.unit == format!("{}-{v}", family(notice.order)));
    let unit = if valid {
        notice.unit.clone()
    } else {
        format!("{}-A", family(notice.order))
    };
    let cell = ExecOrder::ALL
        .iter()
        .position(|o| *o == notice.order)
        .unwrap_or(0);
    // The retained B checkpoint is a rejected indoor counter. Use the shared
    // checkpoint setting while that specific composition remains a review gap.
    let variant = if unit == "ORDER-MILITARIZE-B" {
        "a".into()
    } else {
        unit.chars()
            .last()
            .unwrap_or('A')
            .to_ascii_lowercase()
            .to_string()
    };
    let atlas = format!("policy-bulletins-{variant}");
    let x = (cell % 2) * 768 + 4;
    let y = (cell / 2) as f64 * (1024.0 / 3.0) + 4.0;
    let title = i18n::t(&format!("visual_copy.{unit}.title"));
    let clock = format!(
        "{:02}:{:02}",
        notice.received_minute / 60,
        notice.received_minute % 60
    );
    let day = i18n::fmt_number(f64::from(notice.received_day));
    let received = i18n::tr(
        "policy_bulletin.received",
        Some(&std::collections::BTreeMap::from([
            ("day", day.as_str()),
            ("time", clock.as_str()),
        ])),
    );
    let cfg = crate::game::WeatherConfig::default_config();
    let active = current_activation(gs, notice);
    let on_continue = {
        let cb = p.on_continue.clone();
        Callback::from(move |_| cb.emit(()))
    };
    html! {<section class="policy-bulletin" data-bulletin={notice.id.clone()} data-unit={unit.clone()} aria-labelledby="bulletin-title">
        <StatsBar part={HudPart::Resources} stats={gs.stats.clone()} receipts={gs.receipts.len()} day={gs.day} region={gs.region}/>
        <StatsBar part={HudPart::Conditions} stats={gs.stats.clone()} receipts={gs.receipts.len()} day={gs.day} region={gs.region}
            clock_hour={(gs.continuity.clock_minutes/60) as u8} clock_minute={(gs.continuity.clock_minutes%60) as u8}
            trip_resources={crate::components::ui::leg_summary::render_hud_resources(gs)} pace={Some(gs.pace)} diet={Some(gs.diet)} persona_id={gs.persona_id.clone()}
            exec_order={gs.current_order} policy_readout={gs.current_order.map(|o|stats_bar::policy::readout(gs,o))}
            weather={Some(super::phase::build_weather_badge(gs,&cfg))} weather_readout={Some(stats_bar::weather::readout(gs,&cfg))}
            trip_destination={crate::components::ui::leg_summary::render_hud_destination(gs)}/>
        <figure class="bulletin-illustration">
            <svg viewBox={format!("{x} {y} 760 {}",1024.0/3.0-8.0)} preserveAspectRatio="xMidYMid meet" aria-hidden="true" data-atlas={atlas.clone()} data-cell={cell.to_string()}>
                <image href={crate::paths::asset_path(&format!("static/img/scenes-v2/{atlas}.png"))} width="1536" height="1024"/>
            </svg>
            <figcaption class="scene-caption"><span>{i18n::t("policy_bulletin.label")}</span><h1 id="bulletin-title" ref={heading} tabindex="-1">{title}</h1></figcaption>
        </figure>
        <div class="bulletin-body"><p>{received}</p><p class="bulletin-copy">{i18n::t(&format!("visual_copy.{unit}.activation"))}</p>
            if !active {<p class="bulletin-expired">{i18n::t("policy_bulletin.expired")}{" "}{i18n::t(&format!("visual_copy.{unit}.expiration"))}</p>}
            <button id="bulletin-continue" class="retro-btn-primary" onclick={on_continue}>{i18n::t("ux.continue")}</button>
        </div>
    </section>}
}
