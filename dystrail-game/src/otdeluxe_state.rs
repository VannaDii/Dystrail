//! Oregon Trail Deluxe state scaffolding for parity work.

use serde::{Deserialize, Serialize};

use rand::seq::SliceRandom;

use crate::mechanics::otdeluxe90s::{
    OtDeluxe90sPolicy, OtDeluxeOccupation, OtDeluxePace, OtDeluxeRations, OtDeluxeTrailVariant,
};
use crate::otdeluxe_store::OtDeluxeStoreLineItem;
use crate::state::Season;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeTerrain {
    #[default]
    Plains,
    Mountains,
    #[serde(other)]
    Unknown,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeRouteDecision {
    StayOnTrail,
    SubletteCutoff,
    DallesShortcut,
    RaftColumbia,
    BarlowRoad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeDallesChoice {
    Raft,
    Barlow,
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
pub enum OtDeluxeRiver {
    Kansas,
    BigBlue,
    Green,
    Snake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeCrossingMethod {
    Ford,
    CaulkFloat,
    Ferry,
    Guide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtDeluxeAfflictionKind {
    Illness,
    Injury,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtDeluxeAfflictionOutcome {
    pub member_index: usize,
    pub died: bool,
    pub kind: OtDeluxeAfflictionKind,
    pub disease_id: Option<String>,
    pub display_key: Option<String>,
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
    #[serde(default)]
    pub river_kind: Option<OtDeluxeRiver>,
    #[serde(default)]
    pub computed_miles_today: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OtDeluxeStoreState {
    #[serde(default)]
    pub pending_node: Option<u8>,
    #[serde(default)]
    pub pending_purchase: Option<Vec<OtDeluxeStoreLineItem>>,
    #[serde(default)]
    pub last_node: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxePartyMember {
    pub name: String,
    pub alive: bool,
    pub sick_days_remaining: u8,
    pub injured_days_remaining: u8,
    #[serde(default)]
    pub illness_id: Option<String>,
    #[serde(default)]
    pub injury_id: Option<String>,
}

impl OtDeluxePartyMember {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alive: true,
            sick_days_remaining: 0,
            injured_days_remaining: 0,
            illness_id: None,
            injury_id: None,
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

    #[must_use]
    pub const fn has_affliction(&self) -> bool {
        self.is_sick() || self.is_injured()
    }

    pub fn clear_afflictions(&mut self) {
        self.sick_days_remaining = 0;
        self.injured_days_remaining = 0;
        self.illness_id = None;
        self.injury_id = None;
    }

    pub fn apply_affliction(
        &mut self,
        kind: OtDeluxeAfflictionKind,
        days: u8,
        disease_id: Option<&str>,
    ) -> bool {
        if !self.alive {
            return false;
        }
        if self.has_affliction() {
            self.alive = false;
            self.clear_afflictions();
            return true;
        }
        let days = days.max(1);
        match kind {
            OtDeluxeAfflictionKind::Illness => {
                self.sick_days_remaining = days;
                self.illness_id = disease_id.map(str::to_string);
                self.injury_id = None;
            }
            OtDeluxeAfflictionKind::Injury => {
                self.injured_days_remaining = days;
                self.injury_id = disease_id.map(str::to_string);
                self.illness_id = None;
            }
        }
        false
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
        let mut members = Vec::new();
        for name in names {
            members.push(OtDeluxePartyMember::new(name.into()));
        }
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

    #[must_use]
    pub fn apply_affliction_random<R>(
        &mut self,
        rng: &mut R,
        kind: OtDeluxeAfflictionKind,
        days: u8,
        disease_id: Option<&str>,
    ) -> Option<OtDeluxeAfflictionOutcome>
    where
        R: rand::Rng + ?Sized,
    {
        let mut alive_indices = Vec::new();
        for (idx, member) in self.members.iter().enumerate() {
            if member.alive {
                alive_indices.push(idx);
            }
        }
        let &member_index = alive_indices.choose(rng)?;
        let died = self.members[member_index].apply_affliction(kind, days, disease_id);
        let disease_id = disease_id.map(str::to_string);
        let outcome = OtDeluxeAfflictionOutcome {
            member_index,
            died,
            kind,
            disease_id,
            display_key: None,
        };
        Some(outcome)
    }

    pub fn tick_afflictions(&mut self) {
        for member in &mut self.members {
            if !member.alive {
                continue;
            }
            if member.sick_days_remaining > 0 {
                member.sick_days_remaining = member.sick_days_remaining.saturating_sub(1);
                if member.sick_days_remaining == 0 {
                    member.illness_id = None;
                }
            }
            if member.injured_days_remaining > 0 {
                member.injured_days_remaining = member.injured_days_remaining.saturating_sub(1);
                if member.injured_days_remaining == 0 {
                    member.injury_id = None;
                }
            }
        }
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

impl OtDeluxeCalendar {
    #[must_use]
    pub fn from_day_index(day: u32) -> Self {
        let mut calendar = Self::default();
        if day > 1 {
            calendar.advance_days(day.saturating_sub(1));
        }
        calendar
    }

    #[must_use]
    pub const fn season(&self) -> Season {
        match self.month {
            12 | 1 | 2 => Season::Winter,
            3..=5 => Season::Spring,
            6..=8 => Season::Summer,
            _ => Season::Fall,
        }
    }

    pub fn advance_days(&mut self, mut days: u32) {
        if days == 0 {
            return;
        }
        self.normalize();
        while days > 0 {
            let month_len = Self::month_length(self.year, self.month);
            let remaining = month_len.saturating_sub(self.day_in_month);
            if days <= u32::from(remaining) {
                let days_u8 = u8::try_from(days).unwrap_or(remaining);
                self.day_in_month = self.day_in_month.saturating_add(days_u8);
                return;
            }
            days -= u32::from(remaining).saturating_add(1);
            self.day_in_month = 1;
            if self.month >= 12 {
                self.month = 1;
                self.year = self.year.saturating_add(1);
            } else {
                self.month = self.month.saturating_add(1);
            }
        }
    }

    const fn normalize(&mut self) {
        if self.month == 0 || self.month > 12 {
            self.month = 1;
        }
        let month_len = Self::month_length(self.year, self.month);
        if self.day_in_month == 0 {
            self.day_in_month = 1;
        }
        if self.day_in_month > month_len {
            self.day_in_month = month_len;
        }
    }

    const fn month_length(year: u16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    const fn is_leap_year(year: u16) -> bool {
        (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtDeluxeTravelState {
    pub wagon_state: OtDeluxeWagonState,
    pub delay_days_remaining: u8,
    pub blocked_days_remaining: u8,
    #[serde(default)]
    pub ferry_wait_days_remaining: u8,
    #[serde(default = "default_disease_speed_mult")]
    pub disease_speed_mult: f32,
}

impl Default for OtDeluxeTravelState {
    fn default() -> Self {
        Self {
            wagon_state: OtDeluxeWagonState::default(),
            delay_days_remaining: 0,
            blocked_days_remaining: 0,
            ferry_wait_days_remaining: 0,
            disease_speed_mult: default_disease_speed_mult(),
        }
    }
}

const fn default_disease_speed_mult() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtDeluxeRouteState {
    pub variant: OtDeluxeTrailVariant,
    pub current_node_index: u8,
    pub pending_prompt: Option<OtDeluxeRoutePrompt>,
    #[serde(default)]
    pub dalles_choice: Option<OtDeluxeDallesChoice>,
}

impl Default for OtDeluxeRouteState {
    fn default() -> Self {
        Self {
            variant: OtDeluxeTrailVariant::Main,
            current_node_index: 0,
            pending_prompt: None,
            dalles_choice: None,
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
    #[serde(default, alias = "region")]
    pub terrain: OtDeluxeTerrain,
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
    #[serde(default)]
    pub store: OtDeluxeStoreState,
    pub route: OtDeluxeRouteState,
    pub mods: OtDeluxeModifiers,
}

impl Default for OtDeluxeState {
    fn default() -> Self {
        let calendar = OtDeluxeCalendar::default();
        Self {
            day: 1,
            miles_traveled: 0.0,
            terrain: OtDeluxeTerrain::default(),
            season: calendar.season(),
            calendar,
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
            store: OtDeluxeStoreState::default(),
            route: OtDeluxeRouteState::default(),
            mods: OtDeluxeModifiers::default(),
        }
    }
}

impl OtDeluxeState {
    pub fn advance_days(&mut self, days: u32) {
        if days == 0 {
            return;
        }
        self.day = self.day.saturating_add(days);
        self.calendar.advance_days(days);
        self.season = self.calendar.season();
    }

    #[must_use]
    pub fn effective_oxen(&self, policy: &OtDeluxe90sPolicy) -> f32 {
        self.oxen.effective_oxen(policy.oxen.sick_ox_weight)
    }

    #[must_use]
    pub fn travel_blocked_by_oxen(&self, policy: &OtDeluxe90sPolicy) -> bool {
        self.effective_oxen(policy) < policy.oxen.min_to_move
    }
}

#[cfg(test)]
mod tests {
    use super::{
        OtDeluxeAfflictionKind, OtDeluxeCalendar, OtDeluxeOxenState, OtDeluxePartyMember,
        OtDeluxePartyState, OtDeluxeState,
    };
    use crate::state::Season;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn calendar_advances_and_rolls_year() {
        let mut calendar = OtDeluxeCalendar {
            month: 12,
            day_in_month: 31,
            year: 1848,
        };
        calendar.advance_days(1);
        assert_eq!(calendar.month, 1);
        assert_eq!(calendar.day_in_month, 1);
        assert_eq!(calendar.year, 1849);
    }

    #[test]
    fn calendar_tracks_season_by_month() {
        let spring = OtDeluxeCalendar {
            month: 3,
            ..OtDeluxeCalendar::default()
        };
        assert_eq!(spring.season(), Season::Spring);
        let summer = OtDeluxeCalendar {
            month: 7,
            ..OtDeluxeCalendar::default()
        };
        assert_eq!(summer.season(), Season::Summer);
        let fall = OtDeluxeCalendar {
            month: 10,
            ..OtDeluxeCalendar::default()
        };
        assert_eq!(fall.season(), Season::Fall);
        let winter = OtDeluxeCalendar {
            month: 1,
            ..OtDeluxeCalendar::default()
        };
        assert_eq!(winter.season(), Season::Winter);
    }

    #[test]
    fn state_advancing_updates_calendar_and_season() {
        let mut state = OtDeluxeState::default();
        state.advance_days(31);
        assert_eq!(state.calendar.month, 4);
        assert_eq!(state.calendar.day_in_month, 1);
        assert_eq!(state.season, Season::Spring);
    }

    #[test]
    fn repeat_affliction_kills_member() {
        let mut party = OtDeluxePartyState::from_names(["A"]);
        let died = party.members[0].apply_affliction(OtDeluxeAfflictionKind::Illness, 3, None);
        assert!(!died);
        let died_on_repeat =
            party.members[0].apply_affliction(OtDeluxeAfflictionKind::Injury, 3, None);
        assert!(died_on_repeat);
        assert!(!party.members[0].alive);
    }

    #[test]
    fn apply_affliction_random_targets_alive_member() {
        let mut party = OtDeluxePartyState::from_names(["Ada", "Bob"]);
        let mut rng = SmallRng::seed_from_u64(4);
        let outcome =
            party.apply_affliction_random(&mut rng, OtDeluxeAfflictionKind::Illness, 2, None);
        assert!(outcome.is_some());
    }

    #[test]
    fn random_affliction_targets_alive_member() {
        let mut party = OtDeluxePartyState::from_names(["A", "B"]);
        party.members[0].alive = false;
        let mut rng = SmallRng::seed_from_u64(7);
        let outcome = party
            .apply_affliction_random(&mut rng, OtDeluxeAfflictionKind::Illness, 2, None)
            .expect("alive member");
        assert_eq!(outcome.member_index, 1);
        assert!(party.members[1].is_sick());
    }

    #[test]
    fn tick_afflictions_counts_down() {
        let mut party = OtDeluxePartyState::from_names(["A"]);
        party.members[0].apply_affliction(OtDeluxeAfflictionKind::Illness, 2, None);
        party.tick_afflictions();
        assert_eq!(party.members[0].sick_days_remaining, 1);
    }

    #[test]
    fn affliction_on_dead_member_noops() {
        let mut member = OtDeluxePartyMember::new("A");
        member.alive = false;
        let died = member.apply_affliction(OtDeluxeAfflictionKind::Illness, 2, None);
        assert!(!died);
        assert!(!member.is_sick());
    }

    #[test]
    fn random_affliction_returns_none_when_no_alive_members() {
        let mut party = OtDeluxePartyState::from_names(["A"]);
        party.members[0].alive = false;
        let mut rng = SmallRng::seed_from_u64(9);
        let outcome =
            party.apply_affliction_random(&mut rng, OtDeluxeAfflictionKind::Injury, 2, None);
        assert!(outcome.is_none());
    }

    #[test]
    fn tick_afflictions_clears_ids_when_resolved() {
        let mut party = OtDeluxePartyState::from_names(["A"]);
        party.members[0].sick_days_remaining = 1;
        party.members[0].illness_id = Some(String::from("flu"));
        party.members[0].injured_days_remaining = 1;
        party.members[0].injury_id = Some(String::from("sprain"));
        party.tick_afflictions();
        assert!(party.members[0].illness_id.is_none());
        assert!(party.members[0].injury_id.is_none());
    }

    #[test]
    fn party_counts_reflect_afflictions() {
        let mut party = OtDeluxePartyState::from_names(["A", "B"]);
        party.members[0].apply_affliction(OtDeluxeAfflictionKind::Illness, 2, None);
        party.members[1].apply_affliction(OtDeluxeAfflictionKind::Injury, 2, None);

        assert_eq!(party.alive_count(), 2);
        assert_eq!(party.sick_count(), 1);
        assert_eq!(party.injured_count(), 1);
    }

    #[test]
    fn party_from_names_builds_members() {
        let party = OtDeluxePartyState::from_names(["A", "B", "C"]);
        let names: Vec<_> = party
            .members
            .iter()
            .map(|member| member.name.as_str())
            .collect();
        assert_eq!(names, vec!["A", "B", "C"]);
    }

    #[test]
    fn random_affliction_sets_outcome_for_alive_member() {
        let mut party = OtDeluxePartyState::from_names(["A"]);
        let mut rng = SmallRng::seed_from_u64(5);
        let outcome =
            party.apply_affliction_random(&mut rng, OtDeluxeAfflictionKind::Injury, 2, None);
        assert!(outcome.is_some());
        assert!(party.members[0].is_injured());
    }

    #[test]
    fn oxen_effective_uses_healthy_when_weight_zero() {
        let oxen = OtDeluxeOxenState {
            healthy: 4,
            sick: 2,
        };
        assert!((oxen.effective_oxen(0.0) - 4.0).abs() <= f32::EPSILON);
    }

    #[test]
    fn calendar_from_day_index_advances_dates() {
        let calendar = OtDeluxeCalendar::from_day_index(32);
        assert_eq!(calendar.month, 4);
        assert_eq!(calendar.day_in_month, 1);
    }

    #[test]
    fn tick_afflictions_skips_dead_members() {
        let mut party = OtDeluxePartyState::from_names(["A", "B"]);
        party.members[0].alive = false;
        party.members[1].sick_days_remaining = 2;
        party.tick_afflictions();
        assert_eq!(party.members[1].sick_days_remaining, 1);
    }

    #[test]
    fn calendar_advance_days_noops_on_zero() {
        let mut calendar = OtDeluxeCalendar {
            month: 5,
            day_in_month: 10,
            year: 1848,
        };
        calendar.advance_days(0);
        assert_eq!(calendar.month, 5);
        assert_eq!(calendar.day_in_month, 10);
        assert_eq!(calendar.year, 1848);
    }

    #[test]
    fn state_advance_days_noops_on_zero() {
        let mut state = OtDeluxeState::default();
        let day_before = state.day;
        let month_before = state.calendar.month;
        state.advance_days(0);
        assert_eq!(state.day, day_before);
        assert_eq!(state.calendar.month, month_before);
    }
}
