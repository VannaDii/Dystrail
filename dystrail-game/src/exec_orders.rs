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
    pub const ALL: &'static [ExecOrder] = &[
        ExecOrder::Shutdown,
        ExecOrder::TravelBanLite,
        ExecOrder::BookPanic,
        ExecOrder::TariffTsunami,
        ExecOrder::DoEEliminated,
        ExecOrder::WarDeptReorg,
    ];

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            ExecOrder::Shutdown => "shutdown",
            ExecOrder::TravelBanLite => "travel_ban_lite",
            ExecOrder::BookPanic => "book_panic",
            ExecOrder::TariffTsunami => "tariff_tsunami",
            ExecOrder::DoEEliminated => "doe_eliminated",
            ExecOrder::WarDeptReorg => "war_dept_reorg",
        }
    }

    #[must_use]
    pub const fn name_key(self) -> &'static str {
        match self {
            ExecOrder::Shutdown => "eo.shutdown",
            ExecOrder::TravelBanLite => "eo.travel_ban_lite",
            ExecOrder::BookPanic => "eo.book_panic",
            ExecOrder::TariffTsunami => "eo.tariff_tsunami",
            ExecOrder::DoEEliminated => "eo.doe_eliminated",
            ExecOrder::WarDeptReorg => "eo.war_dept_reorg",
        }
    }
}
