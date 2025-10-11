//! Executive orders and their effects
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_EXEC_ORDERS_DATA: &str =
    include_str!("../../dystrail-web/static/assets/data/exec_orders.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DailyEffect {
    pub sanity: i32,
    pub supplies: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecOrderTier {
    #[serde(default)]
    pub day: u32,
    #[serde(default)]
    pub sanity: i32,
    #[serde(default)]
    pub supplies: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecOrdersConfig {
    #[serde(default)]
    pub orders: HashMap<String, Vec<ExecOrderTier>>,
}

impl ExecOrdersConfig {
    #[must_use]
    pub fn load_from_static() -> Self {
        serde_json::from_str(DEFAULT_EXEC_ORDERS_DATA).unwrap_or_default()
    }

    #[must_use]
    pub fn default_config() -> Self {
        Self::load_from_static()
    }

    #[must_use]
    pub fn effect(&self, order: ExecOrder, day: u32) -> DailyEffect {
        let key = order.key();
        let tiers = self.orders.get(key);
        if let Some(tiers) = tiers {
            let mut effect = DailyEffect::default();
            for tier in tiers {
                if day >= tier.day {
                    effect.sanity = tier.sanity;
                    effect.supplies = tier.supplies;
                } else {
                    break;
                }
            }
            effect
        } else {
            DailyEffect::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecOrder {
    Shutdown,
    Militarize,
    Deregulate,
    TaxCuts,
    Tariffs,
    Gag,
}

impl ExecOrder {
    pub const ALL: &'static [ExecOrder] = &[
        ExecOrder::Shutdown,
        ExecOrder::Militarize,
        ExecOrder::Deregulate,
        ExecOrder::TaxCuts,
        ExecOrder::Tariffs,
        ExecOrder::Gag,
    ];

    #[must_use]
    pub fn effect(&self, cfg: &ExecOrdersConfig, day: u32) -> DailyEffect {
        cfg.effect(*self, day)
    }

    /// Get i18n key for executive order name
    #[must_use]
    pub fn name_key(&self) -> &'static str {
        match self {
            ExecOrder::Shutdown => "exec.shutdown.name",
            ExecOrder::Militarize => "exec.militarize.name",
            ExecOrder::Deregulate => "exec.deregulate.name",
            ExecOrder::TaxCuts => "exec.taxcuts.name",
            ExecOrder::Tariffs => "exec.tariffs.name",
            ExecOrder::Gag => "exec.gag.name",
        }
    }

    #[must_use]
    pub fn key(&self) -> &'static str {
        match self {
            ExecOrder::Shutdown => "shutdown",
            ExecOrder::Militarize => "militarize",
            ExecOrder::Deregulate => "deregulate",
            ExecOrder::TaxCuts => "taxcuts",
            ExecOrder::Tariffs => "tariffs",
            ExecOrder::Gag => "gag",
        }
    }
}
