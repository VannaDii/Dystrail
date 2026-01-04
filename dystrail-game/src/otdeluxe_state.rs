//! Oregon Trail Deluxe state scaffolding for parity work.

use serde::{Deserialize, Serialize};

use crate::mechanics::otdeluxe90s::{
    OtDeluxeOccupation, OtDeluxePace, OtDeluxeRations, OtDeluxeTrailVariant,
};
use crate::state::{Region, Season};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeWagonState {
    #[default]
    Moving,
    Stopped,
    Resting,
    Delayed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeRoutePrompt {
    SubletteCutoff,
    DallesShortcut,
    DallesFinal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeRiverBed {
    Rocky,
    Muddy,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeCrossingMethod {
    Ford,
    CaulkFloat,
    Ferry,
    Guide,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OtDeluxeRiverState {
    pub width_ft: f32,
    pub depth_ft: f32,
    pub swiftness: f32,
    #[serde(default)]
    pub bed: OtDeluxeRiverBed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OtDeluxeCrossingState {
    #[serde(default)]
    pub choice_pending: bool,
    #[serde(default)]
    pub chosen_method: Option<OtDeluxeCrossingMethod>,
    #[serde(default)]
    pub river: Option<OtDeluxeRiverState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxePartyMember {
    pub name: String,
    pub alive: bool,
    pub sick_days_remaining: u8,
    pub injured_days_remaining: u8,
}

impl OtDeluxePartyMember {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alive: true,
            sick_days_remaining: 0,
            injured_days_remaining: 0,
        }
    }

    #[must_use]
    pub const fn is_sick(&self) -> bool {
        self.sick_days_remaining > 0
    }

    #[must_use]
    pub const fn is_injured(&self) -> bool {
        self.injured_days_remaining > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OtDeluxePartyState {
    pub members: Vec<OtDeluxePartyMember>,
}

impl OtDeluxePartyState {
    #[must_use]
    pub fn from_names<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let members = names
            .into_iter()
            .map(|name| OtDeluxePartyMember::new(name.into()))
            .collect();
        Self { members }
    }

    #[must_use]
    pub fn alive_count(&self) -> u16 {
        let count = self.members.iter().filter(|member| member.alive).count();
        u16::try_from(count).unwrap_or(u16::MAX)
    }

    #[must_use]
    pub fn sick_count(&self) -> u16 {
        let count = self
            .members
            .iter()
            .filter(|member| member.is_sick())
            .count();
        u16::try_from(count).unwrap_or(u16::MAX)
    }

    #[must_use]
    pub fn injured_count(&self) -> u16 {
        let count = self
            .members
            .iter()
            .filter(|member| member.is_injured())
            .count();
        u16::try_from(count).unwrap_or(u16::MAX)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OtDeluxeOxenState {
    pub healthy: u16,
    pub sick: u16,
}

impl OtDeluxeOxenState {
    #[must_use]
    pub const fn total(&self) -> u16 {
        self.healthy + self.sick
    }

    #[must_use]
    pub fn effective_oxen(&self, sick_weight: f32) -> f32 {
        if sick_weight <= 0.0 {
            return f32::from(self.healthy);
        }
        f32::from(self.sick).mul_add(sick_weight, f32::from(self.healthy))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OtDeluxeInventory {
    pub food_lbs: u16,
    pub bullets: u16,
    pub clothes_sets: u16,
    pub cash_cents: u32,
    pub spares_wheels: u8,
    pub spares_axles: u8,
    pub spares_tongues: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeCalendar {
    pub month: u8,
    pub day_in_month: u8,
    pub year: u16,
}

impl Default for OtDeluxeCalendar {
    fn default() -> Self {
        Self {
            month: 3,
            day_in_month: 1,
            year: 1848,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OtDeluxeWeatherToday {
    pub temperature_f: i16,
    pub precip_in: f32,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OtDeluxeWeatherState {
    pub today: OtDeluxeWeatherToday,
    pub rain_accum: f32,
    pub snow_depth: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OtDeluxeTravelState {
    pub wagon_state: OtDeluxeWagonState,
    pub delay_days_remaining: u8,
    pub blocked_days_remaining: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeRouteState {
    pub variant: OtDeluxeTrailVariant,
    pub current_node_index: u8,
    pub pending_prompt: Option<OtDeluxeRoutePrompt>,
}

impl Default for OtDeluxeRouteState {
    fn default() -> Self {
        Self {
            variant: OtDeluxeTrailVariant::Main,
            current_node_index: 0,
            pending_prompt: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OtDeluxeModifiers {
    pub occupation: Option<OtDeluxeOccupation>,
    pub exec_orders_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeState {
    pub day: u32,
    pub miles_traveled: f32,
    pub region: Region,
    pub season: Season,
    pub calendar: OtDeluxeCalendar,
    pub party: OtDeluxePartyState,
    pub health_general: u16,
    pub death_imminent_days_remaining: u8,
    pub general_strain: f32,
    pub oxen: OtDeluxeOxenState,
    pub inventory: OtDeluxeInventory,
    pub pace: OtDeluxePace,
    pub rations: OtDeluxeRations,
    pub weather: OtDeluxeWeatherState,
    pub travel: OtDeluxeTravelState,
    pub crossing: OtDeluxeCrossingState,
    pub route: OtDeluxeRouteState,
    pub mods: OtDeluxeModifiers,
}

impl Default for OtDeluxeState {
    fn default() -> Self {
        Self {
            day: 1,
            miles_traveled: 0.0,
            region: Region::Heartland,
            season: Season::default(),
            calendar: OtDeluxeCalendar::default(),
            party: OtDeluxePartyState::default(),
            health_general: 0,
            death_imminent_days_remaining: 0,
            general_strain: 0.0,
            oxen: OtDeluxeOxenState::default(),
            inventory: OtDeluxeInventory::default(),
            pace: OtDeluxePace::Steady,
            rations: OtDeluxeRations::Filling,
            weather: OtDeluxeWeatherState::default(),
            travel: OtDeluxeTravelState::default(),
            crossing: OtDeluxeCrossingState::default(),
            route: OtDeluxeRouteState::default(),
            mods: OtDeluxeModifiers::default(),
        }
    }
}
