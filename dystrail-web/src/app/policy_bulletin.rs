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
