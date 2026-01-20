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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExecOrderEffects {
    pub travel_multiplier: f32,
    pub breakdown_bonus: f32,
    pub encounter_delta: f32,
    pub strain_bonus: f32,
    pub supplies_delta: i32,
    pub sanity_delta: i32,
    pub morale_delta: i32,
}

impl Default for ExecOrderEffects {
    fn default() -> Self {
        Self {
            travel_multiplier: 1.0,
            breakdown_bonus: 0.0,
            encounter_delta: 0.0,
            strain_bonus: 0.0,
            supplies_delta: 0,
            sanity_delta: 0,
            morale_delta: 0,
        }
    }
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
