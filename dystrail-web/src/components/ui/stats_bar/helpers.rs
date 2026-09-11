use crate::game::{exec_orders::ExecOrder, weather::Weather};
#[must_use]
pub const fn weather_symbol(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "☼",
        Weather::Storm => "⛈",
        Weather::HeatWave => "☀",
        Weather::ColdSnap => "❄",
        Weather::Smoke => "☁",
    }
}

pub(super) const fn weather_sprite_class(weather: Weather) -> &'static str {
    match weather {
        Weather::Clear => "sprite-weather-clear",
        Weather::Storm => "sprite-weather-storm",
        Weather::HeatWave => "sprite-weather-heat",
        Weather::ColdSnap => "sprite-weather-cold",
        Weather::Smoke => "sprite-weather-smoke",
    }
}

pub(super) const fn exec_sprite_class(order: ExecOrder) -> &'static str {
    match order {
        ExecOrder::Shutdown => "sprite-eo-shutdown",
        ExecOrder::TravelBanLite => "sprite-eo-travelban",
        ExecOrder::BookPanic => "sprite-eo-book",
        ExecOrder::TariffTsunami => "sprite-eo-tariff",
        ExecOrder::DoEEliminated => "sprite-eo-doe",
        ExecOrder::WarDeptReorg => "sprite-eo-war",
    }
}
