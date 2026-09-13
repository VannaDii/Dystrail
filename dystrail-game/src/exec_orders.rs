//! Executive orders: definitions and metadata.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecOrder {
    Shutdown,
    TravelBanLite,
    BookPanic,
    TariffTsunami,
    DoEEliminated,
    WarDeptReorg,
}

impl ExecOrder {
    pub const ALL: &'static [Self] = &[
        Self::Shutdown,
        Self::TravelBanLite,
        Self::BookPanic,
        Self::TariffTsunami,
        Self::DoEEliminated,
        Self::WarDeptReorg,
    ];

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Shutdown => "shutdown",
            Self::TravelBanLite => "travel_ban_lite",
            Self::BookPanic => "book_panic",
            Self::TariffTsunami => "tariff_tsunami",
            Self::DoEEliminated => "doe_eliminated",
            Self::WarDeptReorg => "war_dept_reorg",
        }
    }

    #[must_use]
    pub const fn name_key(self) -> &'static str {
        match self {
            Self::Shutdown => "eo.shutdown",
            Self::TravelBanLite => "eo.travel_ban_lite",
            Self::BookPanic => "eo.book_panic",
            Self::TariffTsunami => "eo.tariff_tsunami",
            Self::DoEEliminated => "eo.doe_eliminated",
            Self::WarDeptReorg => "eo.war_dept_reorg",
        }
    }
}

/// Per-day policy effects. Both simulation and the HUD use this rule description.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DailyEffect {
    pub supplies: i32,
    pub sanity: i32,
    pub morale: i32,
    pub travel_multiplier: f32,
    pub breakdown_bonus: f32,
}
impl ExecOrder {
    #[must_use]
    pub const fn daily_effect(self, morale: i32, legal_fund: bool) -> DailyEffect {
        let mut effect = DailyEffect {
            supplies: 0,
            sanity: 0,
            morale: 0,
            travel_multiplier: 1.0,
            breakdown_bonus: 0.0,
        };
        match self {
            Self::Shutdown => {
                effect.morale = -1;
                effect.supplies = -1;
            }
            Self::TravelBanLite => {
                effect.sanity = -1;
                effect.travel_multiplier = crate::constants::EXEC_ORDER_SPEED_BONUS;
            }
            Self::BookPanic => {
                effect.sanity = if morale < 7 { -1 } else { 0 };
            }
            Self::TariffTsunami => {
                effect.supplies = if legal_fund { 0 } else { -1 };
            }
            Self::DoEEliminated => {
                effect.morale = -1;
            }
            Self::WarDeptReorg => {
                effect.breakdown_bonus = crate::constants::EXEC_ORDER_BREAKDOWN_BONUS;
            }
        }
        effect
    }
}
