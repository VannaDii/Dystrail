use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecOrder {
    Shutdown,
    TravelBanLite,
    GasStovePolice,
    BookPanic,
    DeportationSweep,
    TariffTsunami,
    DoEEliminated,
    WarDeptReorg,
}

impl ExecOrder {
    pub const ALL: [ExecOrder; 8] = [
        ExecOrder::Shutdown,
        ExecOrder::TravelBanLite,
        ExecOrder::GasStovePolice,
        ExecOrder::BookPanic,
        ExecOrder::DeportationSweep,
        ExecOrder::TariffTsunami,
        ExecOrder::DoEEliminated,
        ExecOrder::WarDeptReorg,
    ];

    #[must_use]
    pub fn name_key(self) -> &'static str {
        match self {
            ExecOrder::Shutdown => "eo.shutdown",
            ExecOrder::TravelBanLite => "eo.travel-ban-lite",
            ExecOrder::GasStovePolice => "eo.gas-stove-police",
            ExecOrder::BookPanic => "eo.book-panic",
            ExecOrder::DeportationSweep => "eo.deportation-sweep",
            ExecOrder::TariffTsunami => "eo.tariff-tsunami",
            ExecOrder::DoEEliminated => "eo.doe-eliminated",
            ExecOrder::WarDeptReorg => "eo.war-dept-reorg",
        }
    }

    /// Apply per-day global modifiers. UI is responsible for localization of any messages.
    pub fn apply_daily(self, _day: u32, supplies_cost: &mut i32, sanity_cost: &mut i32) {
        match self {
            ExecOrder::Shutdown | ExecOrder::DeportationSweep => {
                *sanity_cost += 1;
            }
            ExecOrder::TravelBanLite | ExecOrder::TariffTsunami => {
                *supplies_cost += 1;
            }
            ExecOrder::GasStovePolice
            | ExecOrder::BookPanic
            | ExecOrder::DoEEliminated
            | ExecOrder::WarDeptReorg => {}
        }
    }
}
