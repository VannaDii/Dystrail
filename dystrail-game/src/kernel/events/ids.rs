use crate::journey::EventKind;

/// Deterministic event code for i18n and UI rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelEventCode {
    LegacyLog,
    WeatherResolved,
    DailyConsumptionApplied,
    HealthTickApplied,
    GeneralStrainComputed,
    ExecOrderStarted,
    ExecOrderEnded,
    BreakdownStarted,
    BreakdownResolved,
    EncounterTriggered,
    RandomEventResolved,
    TradeResolved,
    HuntResolved,
    AfflictionTriggered,
    NavigationEvent,
    CrossingResolved,
    TravelBlocked,
}

impl KernelEventCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LegacyLog => "event.legacy.log",
            Self::WeatherResolved => "event.weather.resolved",
            Self::DailyConsumptionApplied => "event.supplies.daily_consumption_applied",
            Self::HealthTickApplied => "event.health.tick_applied",
            Self::GeneralStrainComputed => "event.health.general_strain_computed",
            Self::ExecOrderStarted => "event.exec_order.started",
            Self::ExecOrderEnded => "event.exec_order.ended",
            Self::BreakdownStarted => "event.vehicle.breakdown_started",
            Self::BreakdownResolved => "event.vehicle.breakdown_resolved",
            Self::EncounterTriggered => "event.encounter.triggered",
            Self::RandomEventResolved => "event.random.resolved",
            Self::TradeResolved => "event.trade.resolved",
            Self::HuntResolved => "event.hunt.resolved",
            Self::AfflictionTriggered => "event.affliction.triggered",
            Self::NavigationEvent => "event.navigation.resolved",
            Self::CrossingResolved => "event.crossing.resolved",
            Self::TravelBlocked => "event.travel.blocked",
        }
    }
}

impl From<&EventKind> for KernelEventCode {
    fn from(value: &EventKind) -> Self {
        match value {
            EventKind::LegacyLogKey => Self::LegacyLog,
            EventKind::WeatherResolved => Self::WeatherResolved,
            EventKind::DailyConsumptionApplied => Self::DailyConsumptionApplied,
            EventKind::HealthTickApplied => Self::HealthTickApplied,
            EventKind::GeneralStrainComputed => Self::GeneralStrainComputed,
            EventKind::ExecOrderStarted => Self::ExecOrderStarted,
            EventKind::ExecOrderEnded => Self::ExecOrderEnded,
            EventKind::BreakdownStarted => Self::BreakdownStarted,
            EventKind::BreakdownResolved => Self::BreakdownResolved,
            EventKind::EncounterTriggered => Self::EncounterTriggered,
            EventKind::RandomEventResolved => Self::RandomEventResolved,
            EventKind::TradeResolved => Self::TradeResolved,
            EventKind::HuntResolved => Self::HuntResolved,
            EventKind::AfflictionTriggered => Self::AfflictionTriggered,
            EventKind::NavigationEvent => Self::NavigationEvent,
            EventKind::CrossingResolved => Self::CrossingResolved,
            EventKind::TravelBlocked => Self::TravelBlocked,
        }
    }
}
