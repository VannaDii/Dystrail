use crate::game::{
    CrossingConfig, CrossingKind, GameState, calculate_bribe_cost, can_afford_bribe, can_use_permit,
};
use crate::i18n;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossingViewModel {
    pub title: String,
    pub prompt: String,
    pub detour_label: String,
    pub detour_desc: String,
    pub bribe_label: String,
    pub bribe_desc: String,
    pub permit_label: String,
    pub permit_desc: String,
    pub back_label: String,
    pub permit_available: bool,
    pub bribe_available: bool,
    pub shutdown_notice: Option<String>,
}

/// Build view model with resolved strings and availability flags
pub fn build_crossing_viewmodel(
    gs: &GameState,
    cfg: &CrossingConfig,
    kind: CrossingKind,
) -> Result<CrossingViewModel, String> {
    let type_cfg = cfg
        .types
        .get(&kind)
        .ok_or_else(|| format!("Unknown crossing type: {kind:?}"))?;

    let title = match kind {
        CrossingKind::Checkpoint => i18n::t("cross.types.checkpoint"),
        CrossingKind::BridgeOut => i18n::t("cross.types.bridge_out"),
    };
    let prompt = i18n::t("cross.prompt");

    let mut detour_days = type_cfg.detour.days;
    let mut detour_pants = type_cfg.detour.pants;

    if let Some(weather_mod) = cfg.global_mods.weather.get(&gs.weather_state.today) {
        if let Some(extra_days) = weather_mod.detour.days {
            detour_days += extra_days;
        }
        if let Some(extra_pants) = weather_mod.detour.pants {
            detour_pants += extra_pants;
        }
    }

    let days_str = if detour_days >= 0 {
        format!("+{detour_days}")
    } else {
        detour_days.to_string()
    };
    let supplies_str = if type_cfg.detour.supplies >= 0 {
        format!("+{supplies}", supplies = type_cfg.detour.supplies)
    } else {
        type_cfg.detour.supplies.to_string()
    };
    let pants_str = if detour_pants >= 0 {
        format!("+{detour_pants}")
    } else {
        detour_pants.to_string()
    };

    let mut detour_args = std::collections::BTreeMap::new();
    detour_args.insert("days", days_str.as_str());
    detour_args.insert("supplies", supplies_str.as_str());
    detour_args.insert("pants", pants_str.as_str());
    let detour_label = i18n::tr("cross.options.detour", Some(&detour_args));
    let detour_desc = i18n::t("cross.desc.detour");

    let bribe_cost_cents =
        calculate_bribe_cost(type_cfg.bribe.base_cost_cents, gs.mods.bribe_discount_pct);
    let bribe_cost_display = format_currency(bribe_cost_cents);
    let mut bribe_args = std::collections::BTreeMap::new();
    bribe_args.insert("cost", bribe_cost_display.as_str());
    let bribe_label = i18n::tr("cross.options.bribe", Some(&bribe_args));
    let bribe_desc = i18n::t("cross.desc.bribe");

    let permit_label = i18n::t("cross.options.permit");
    let permit_desc = i18n::t("cross.desc.permit");
    let back_label = i18n::t("cross.options.back");

    let permit_available = can_use_permit(gs, &kind);
    let bribe_available = can_afford_bribe(gs, cfg, kind);

    let shutdown_notice = cfg
        .global_mods
        .exec_orders
        .get("Shutdown")
        .and_then(|exec_mod| {
            if matches!(
                gs.current_order,
                Some(crate::game::exec_orders::ExecOrder::Shutdown)
            ) {
                let chance_pct = (exec_mod.bribe_success_chance * 100.0)
                    .round()
                    .clamp(0.0, 100.0);
                let mut args = std::collections::BTreeMap::new();
                let chance_str = format!("{chance_pct:.0}");
                args.insert("chance", chance_str.as_str());
                Some(i18n::tr("cross.policy.shutdown", Some(&args)))
            } else {
                None
            }
        });

    Ok(CrossingViewModel {
        title,
        prompt,
        detour_label,
        detour_desc,
        bribe_label,
        bribe_desc,
        permit_label,
        permit_desc,
        back_label,
        permit_available,
        bribe_available,
        shutdown_notice,
    })
}

fn format_currency(cents: i64) -> String {
    crate::i18n::fmt_currency(cents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::exec_orders::ExecOrder;
    use crate::game::weather::Weather;

    #[test]
    fn build_crossing_viewmodel_reports_missing_type() {
        crate::i18n::set_lang("en");
        let mut cfg = CrossingConfig::default();
        cfg.types.clear();
        let gs = GameState::default();
        let err = build_crossing_viewmodel(&gs, &cfg, CrossingKind::Checkpoint)
            .expect_err("missing type should error");
        assert!(err.contains("Unknown crossing type"));
    }

    #[test]
    fn build_crossing_viewmodel_applies_mods_and_availability() {
        crate::i18n::set_lang("en");
        let mut gs = GameState::default();
        gs.weather_state.today = Weather::Storm;
        gs.current_order = Some(ExecOrder::Shutdown);
        gs.inventory.tags.insert(String::from("permit"));
        gs.budget_cents = 10_000;

        let cfg = CrossingConfig::default();
        let vm = build_crossing_viewmodel(&gs, &cfg, CrossingKind::Checkpoint)
            .expect("view model should build");

        assert!(vm.permit_available);
        assert!(vm.bribe_available);
        assert!(vm.shutdown_notice.is_some());
        assert!(vm.detour_label.contains('+'));
    }

    #[test]
    fn build_crossing_viewmodel_formats_negative_detour_values() {
        crate::i18n::set_lang("en");
        let gs = GameState::default();
        let mut cfg = CrossingConfig::default();
        if let Some(kind_cfg) = cfg.types.get_mut(&CrossingKind::BridgeOut) {
            kind_cfg.detour.days = -1;
            kind_cfg.detour.supplies = 3;
            kind_cfg.detour.pants = -2;
        }

        let vm = build_crossing_viewmodel(&gs, &cfg, CrossingKind::BridgeOut)
            .expect("view model should build");
        assert!(vm.detour_label.contains("-1"));
        assert!(vm.detour_label.contains("+3"));
        assert!(vm.detour_label.contains("-2"));
    }
}
