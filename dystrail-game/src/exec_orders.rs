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
