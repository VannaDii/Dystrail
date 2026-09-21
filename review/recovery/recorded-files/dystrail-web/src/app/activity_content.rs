//! Saved offers and completed activity copy share one presentation-only identity.
use crate::{game::{GameState,activities::Activity},i18n};
use super::{state::AppState,Phase};
use yew::prelude::*;

pub fn occurrence(gs:&GameState,action:Activity)->String {
    if Activity::TOWN.contains(&action) {
        format!("work/{}/{}",gs.continuity.route_services.route_id,gs.continuity.route_services.stop.unwrap_or(0))
    } else {
        format!("gather/{:?}/{}",gs.region,gs.continuity.activities.foraged_on.unwrap_or(0))
    }
}

pub fn unit(gs:&GameState,action:Activity)->String {
    super::visual_content::selected(gs,super::workshop::activity(action),&occurrence(gs,action))
}

pub fn text(unit:&str,field:&str)->String {
    i18n::t(&format!("visual_copy.{unit}.{field}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rest_selection_preserves_daily_settlement_and_survives_import() {
        for supplies in [0, 8] {
            for seed in 0..12 {
                let mut original = GameState { seed, day:5, ..GameState::default() };
                original.stats.supplies = supplies;
                original.stats.hp = 6;
                original.stats.sanity = 4;
                original.continuity.visual_content.edition = super::super::visual_content::EDITION;
                let occurrence = rest_occurrence(&original);
                let mut illustrated = original.clone();
                let unit = super::super::visual_content::select(&mut illustrated, "ACT-REST", &occurrence);
                let mut restored: GameState = serde_json::from_value(serde_json::to_value(&illustrated).unwrap()).unwrap();
                assert_eq!(rest_unit(&restored), unit);
                let config = crate::game::CampConfig::default_config();
                assert!(crate::game::camp_rest(&mut original, &config).rested);
                assert!(crate::game::camp_rest(&mut restored, &config).rested);
                let expected = serde_json::to_value(&original).unwrap();
                let mut actual = serde_json::to_value(&restored).unwrap();
                actual["visual_content"] = expected["visual_content"].clone();
                assert_eq!(actual, expected);
                assert_eq!(restored.continuity.visual_content.selections[&format!("ACT-REST/{occurrence}")], unit);
                assert!(!crate::game::camp_rest(&mut restored, &config).rested);
            }
        }
    }
    #[test]
    fn barter_scenes_preserve_atomic_exchange_and_saved_inventory() {
        for kind in 0..3 {
            let family = super::super::workshop::trade(kind);
            let mut original = GameState::default();
            original.stats.supplies = 8;
            original.inventory.spares.tire = 1;
            original.continuity.route_services.stop = Some(160);
            let occurrence = trade_occurrence(&original);
            assert_eq!(trade_unit(&original, kind), format!("{family}-A"));
            original.continuity.visual_content.edition = super::super::visual_content::EDITION;
            let variants: std::collections::BTreeSet<_> = (0..60).map(|seed| {
                original.seed = seed;
                trade_unit(&original, kind)
            }).collect();
            assert_eq!(variants.len(), 3);
            let mut illustrated = original.clone();
            let unit = super::super::visual_content::select(&mut illustrated, family, &occurrence);
            let mut restored: GameState = serde_json::from_value(serde_json::to_value(&illustrated).unwrap()).unwrap();
            assert_eq!(trade_unit(&restored, kind), unit);
            assert!(original.trade_route_offer(kind));
            assert!(restored.trade_route_offer(kind));
            let before = original.clone();
            original.advance_clock(&before, 30);
            let before = restored.clone();
            restored.advance_clock(&before, 30);
            let expected = serde_json::to_value(&original).unwrap();
            let mut actual = serde_json::to_value(&restored).unwrap();
            actual["visual_content"] = expected["visual_content"].clone();
            assert_eq!(actual, expected);
            for other in 0..3 { assert!(!restored.trade_route_offer(other)); }
            assert_eq!(restored.vehicle.health, original.vehicle.health);
        }
    }

    #[test]
    fn saved_offers_do_not_change_activity_costs_or_random_streams() {
        for action in Activity::ROADSIDE.into_iter().chain(Activity::TOWN) {
            for clock in [600, 1190] {
                let mut original = GameState::default();
                original.stats.supplies = 8;
                original.stats.hp = 8;
                original.stats.sanity = 8;
                original.continuity.clock_minutes = clock;
                original.continuity.visual_content.edition = super::super::visual_content::EDITION;
                if Activity::TOWN.contains(&action) {
                    original.continuity.route_services.stop = Some(160);
                }
                let mut illustrated = original.clone();
                let occurrence = occurrence(&illustrated, action);
                let selected = super::super::visual_content::select(&mut illustrated, super::super::workshop::activity(action), &occurrence);
                let restored: GameState = serde_json::from_value(serde_json::to_value(&illustrated).unwrap()).unwrap();
                assert_eq!(unit(&restored, action), selected);
                assert!(original.perform_activity(action));
                assert!(illustrated.perform_activity(action));
                let expected = serde_json::to_value(&original).unwrap();
                let mut actual = serde_json::to_value(&illustrated).unwrap();
                actual["visual_content"] = expected["visual_content"].clone();
                assert_eq!(actual, expected);
                assert!(!illustrated.perform_activity(action));
            }
        }
    }

    #[test]
    fn all_activity_variants_are_reachable_and_legacy_offers_stay_put() {
        for action in Activity::ROADSIDE.into_iter().chain(Activity::TOWN) {
            let mut gs = GameState::default();
            let family = super::super::workshop::activity(action);
            let occurrence = occurrence(&gs, action);
            assert_eq!(super::super::visual_content::select(&mut gs, family, &occurrence), format!("{family}-A"));
            gs.continuity.visual_content.edition = super::super::visual_content::EDITION;
            assert_eq!(unit(&gs, action), format!("{family}-A"));
            gs.continuity.visual_content.selections.clear();
            let variants: std::collections::BTreeSet<_> = (0..60).map(|seed| {
                gs.seed=seed; unit(&gs, action)
            }).collect();
            assert_eq!(variants.len(),3);
        }
    }
}

/// Seal visible offers before the player acts. A reload or menu visit cannot
/// choose again, and browsing never touches the engine's random streams.
#[hook]
pub fn use_offers(app:&AppState) {
    let offers=app.session.as_ref().filter(|_|*app.recovery_ready
        && matches!(*app.phase,Phase::Camp|Phase::Town)
        && app.aftermath.is_none() && app.pending_turn.is_none())
        .map(|session| {
            let gs=session.state();
            let actions=if gs.continuity.route_services.stop.is_some(){Activity::TOWN}else{Activity::ROADSIDE};
            actions.into_iter().filter_map(|action| {
                let family=super::workshop::activity(action);
                let occurrence=occurrence(gs,action);
                (!gs.continuity.visual_content.selections.contains_key(&format!("{family}/{occurrence}")))
                    .then(||(family.to_owned(),occurrence))
            }).collect::<Vec<_>>()
        }).unwrap_or_default();
    let handle=app.session.clone();
    use_effect_with(offers,move|offers| {
        if !offers.is_empty() && let Some(mut session)=(*handle).clone() {
            session.with_state_mut(|gs|for (family,occurrence) in offers {
                super::visual_content::select(gs,family,occurrence);
            });
            handle.set(Some(session));
        }
    });
}
