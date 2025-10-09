//! Executive orders and their effects
use serde::{Deserialize, Serialize};

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

    pub fn apply_daily(&self, _day: u32, supplies_cost: &mut i32, sanity_cost: &mut i32) {
        match self {
            ExecOrder::Shutdown | ExecOrder::TaxCuts => {
                *supplies_cost += 1;
                *sanity_cost += 1;
            }
            ExecOrder::Militarize => {
                *supplies_cost += 2;
                *sanity_cost += 1;
            }
            ExecOrder::Deregulate => {
                *supplies_cost += 1;
                *sanity_cost += 2;
            }
            ExecOrder::Tariffs => {
                *supplies_cost += 3;
                *sanity_cost += 1;
            }
            ExecOrder::Gag => {
                *supplies_cost += 1;
                *sanity_cost += 3;
            }
        }
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
}
