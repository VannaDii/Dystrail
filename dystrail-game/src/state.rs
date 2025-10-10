use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::camp::CampState;
use crate::data::{Encounter, EncounterData};
use crate::encounters::pick_encounter;
use crate::exec_orders::ExecOrder;
use crate::personas::{Persona, PersonaMods};
use crate::vehicle::{Breakdown, Part, Vehicle};
use crate::weather::{WeatherConfig, WeatherState};

/// Default pace setting
fn default_pace() -> String {
    "steady".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakdown_consumes_spare_and_clears_block() {
        let mut state = GameState::default();
        state.inventory.spares.tire = 1;
        state.breakdown = Some(Breakdown {
            part: Part::Tire,
            day_started: 1,
        });
        state.travel_blocked = true;
        state.rng = Some(ChaCha20Rng::seed_from_u64(1));
        state.data = Some(EncounterData::empty());

        let (_ended, _msg, _started) = state.travel_next_leg();

        assert_eq!(state.inventory.spares.tire, 0);
        assert!(!state.travel_blocked);
        assert!(state.breakdown.is_none());
    }

    #[test]
    fn breakdown_without_spare_resolves_after_stall() {
        let mut state = GameState::default();
        state.breakdown = Some(Breakdown {
            part: Part::Battery,
            day_started: 1,
        });
        state.travel_blocked = true;
        state.rng = Some(ChaCha20Rng::seed_from_u64(2));
        state.data = Some(EncounterData::empty());

        let (_ended_first, msg_first, _started_first) = state.travel_next_leg();
        assert_eq!(msg_first, "log.travel-blocked");
        assert!(state.travel_blocked);
        assert!(state.breakdown.is_some());

        let (_ended_second, msg_second, _started_second) = state.travel_next_leg();
        assert_eq!(msg_second, "log.traveled");
        assert!(!state.travel_blocked);
        assert!(state.breakdown.is_none());
    }

    #[test]
    fn exec_order_drain_clamped_to_zero() {
        let mut state = GameState::default();
        state.stats.supplies = 0;
        state.stats.sanity = 0;
        state.rng = Some(ChaCha20Rng::seed_from_u64(3));
        state.encounter_chance_today = 0.0;
        state.data = Some(EncounterData::empty());

        let (_ended, _msg, _started) = state.travel_next_leg();

        assert!(state.stats.supplies >= 0, "supplies went negative");
        assert!(state.stats.sanity >= 0, "sanity went negative");
    }
}

/// Default diet setting
fn default_diet() -> String {
    "mixed".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    Classic,
    Deep,
}

impl GameMode {
    #[must_use]
    pub fn is_deep(self) -> bool {
        matches!(self, GameMode::Deep)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    Heartland,
    RustBelt,
    Beltway,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub supplies: i32,
    pub hp: i32,
    pub sanity: i32,
    pub credibility: i32,
    pub morale: i32,
    pub allies: i32,
    pub pants: i32, // 0..100
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            supplies: 10,
            hp: 10,
            sanity: 10,
            credibility: 5,
            morale: 5,
            allies: 0,
            pants: 0,
        }
    }
}

impl Stats {
    pub fn clamp(&mut self) {
        self.hp = self.hp.clamp(0, 10);
        self.sanity = self.sanity.clamp(0, 10);
        self.credibility = self.credibility.clamp(0, 20);
        self.morale = self.morale.clamp(0, 10);
        self.supplies = self.supplies.clamp(0, 20);
        self.allies = self.allies.clamp(0, 50);
        self.pants = self.pants.clamp(0, 100);
    }
}

/// Player inventory including spares and tags
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Inventory {
    #[serde(default)]
    pub spares: Spares,
    #[serde(default)]
    pub tags: HashSet<String>,
}

/// Vehicle and equipment spares
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Spares {
    #[serde(default)]
    pub tire: i32,
    #[serde(default)]
    pub battery: i32,
    #[serde(default)]
    pub alt: i32, // alternator
    #[serde(default)]
    pub pump: i32, // fuel pump
}

/// Party configuration (leader plus four companions)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Party {
    #[serde(default)]
    pub leader: String,
    #[serde(default)]
    pub companions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Boot,
    Persona,
    Menu,
    Travel,
    Encounter,
    Boss,
    Result,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub mode: GameMode,
    pub seed: u64,
    pub day: u32,
    pub region: Region,
    pub stats: Stats,
    #[serde(default)]
    pub budget: i32,
    /// Budget in cents for precise calculations
    #[serde(default)]
    pub budget_cents: i64,
    #[serde(default)]
    pub inventory: Inventory,
    #[serde(default)]
    pub persona_id: Option<String>,
    #[serde(default)]
    pub score_mult: f32,
    #[serde(default)]
    pub mods: PersonaMods,
    #[serde(default)]
    pub party: Party,
    /// Current pace setting
    #[serde(default = "default_pace")]
    pub pace: String,
    /// Current info diet setting
    #[serde(default = "default_diet")]
    pub diet: String,
    /// Calculated receipt finding bonus percentage for this tick
    #[serde(default)]
    pub receipt_bonus_pct: i32,
    /// Base encounter chance for today after pace modifiers
    #[serde(default)]
    pub encounter_chance_today: f32,
    /// Whether an encounter has already occurred on the current day
    #[serde(default)]
    pub encounter_occurred_today: bool,
    /// Distance multiplier for today
    #[serde(default)]
    pub distance_today: f32,
    pub logs: Vec<String>,
    pub receipts: Vec<String>,
    pub current_encounter: Option<Encounter>,
    pub current_order: ExecOrder,
    /// Vehicle state and spares
    #[serde(default)]
    pub vehicle: Vehicle,
    /// Active breakdown blocking travel
    #[serde(default)]
    pub breakdown: Option<Breakdown>,
    /// Whether travel is blocked due to breakdown
    #[serde(default)]
    pub travel_blocked: bool,
    /// Weather state and history for streak tracking
    #[serde(default)]
    pub weather_state: WeatherState,
    /// Camp state and cooldowns
    #[serde(default)]
    pub camp: CampState,
    #[serde(skip)]
    pub rng: Option<ChaCha20Rng>,
    #[serde(skip)]
    pub data: Option<EncounterData>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            mode: GameMode::Classic,
            seed: 0,
            day: 1,
            region: Region::Heartland,
            stats: Stats::default(),
            budget: 100,
            budget_cents: 10_000, // $100.00 in cents
            inventory: Inventory::default(),
            persona_id: None,
            score_mult: 1.0,
            mods: PersonaMods::default(),
            party: Party::default(),
            pace: default_pace(),
            diet: default_diet(),
            receipt_bonus_pct: 0,
            encounter_chance_today: 0.35,
            encounter_occurred_today: false,
            distance_today: 1.0,
            logs: vec![String::from("log.booting")],
            receipts: vec![],
            current_encounter: None,
            current_order: ExecOrder::Shutdown,
            vehicle: Vehicle::default(),
            breakdown: None,
            travel_blocked: false,
            weather_state: WeatherState::default(),
            camp: CampState::default(),
            rng: None,
            data: None,
        }
    }
}

impl GameState {
    fn seed_bytes(s: u64) -> [u8; 32] {
        #[inline]
        fn b(x: u64, shift: u8, xorv: u8) -> u8 {
            (((x >> shift) & 0xFF) as u8) ^ xorv
        }
        [
            b(s, 56, 0x00),
            b(s, 48, 0x00),
            b(s, 40, 0x00),
            b(s, 32, 0x00),
            b(s, 24, 0x00),
            b(s, 16, 0x00),
            b(s, 8, 0x00),
            b(s, 0, 0x00),
            b(s, 56, 0xAA),
            b(s, 48, 0x55),
            b(s, 40, 0xAA),
            b(s, 32, 0x55),
            b(s, 24, 0xAA),
            b(s, 16, 0x55),
            b(s, 8, 0xAA),
            b(s, 0, 0x55),
            b(s, 56, 0x11),
            b(s, 48, 0x22),
            b(s, 40, 0x33),
            b(s, 32, 0x44),
            b(s, 24, 0x55),
            b(s, 16, 0x66),
            b(s, 8, 0x77),
            b(s, 0, 0x88),
            b(s, 56, 0x99),
            b(s, 48, 0xAA),
            b(s, 40, 0xBB),
            b(s, 32, 0xCC),
            b(s, 24, 0xDD),
            b(s, 16, 0xEE),
            b(s, 8, 0xFF),
            b(s, 0, 0x10),
        ]
    }

    #[must_use]
    pub fn with_seed(mut self, seed: u64, mode: GameMode, data: EncounterData) -> Self {
        let bytes = Self::seed_bytes(seed);
        self.mode = mode;
        self.seed = seed;
        self.rng = Some(ChaCha20Rng::from_seed(bytes));
        self.logs.push(String::from("log.seed-set"));
        self.data = Some(data);
        self
    }

    #[must_use]
    pub fn rehydrate(mut self, data: EncounterData) -> Self {
        let bytes = Self::seed_bytes(self.seed);
        self.rng = Some(ChaCha20Rng::from_seed(bytes));
        self.data = Some(data);
        self
    }

    #[must_use]
    pub fn region_by_day(day: u32) -> Region {
        match day {
            0..=4 => Region::Heartland,
            5..=9 => Region::RustBelt,
            _ => Region::Beltway,
        }
    }

    pub fn travel_next_leg(&mut self) -> (bool, String, bool) {
        // Step 1: Apply Pace & Diet deltas (handled externally via apply_pace_and_diet call)

        // Step 2: Apply Weather effects
        self.apply_weather_effects();

        // Step 3: Vehicle breakdown roll
        let breakdown_started = self.vehicle_roll();
        self.resolve_breakdown();

        // Step 4: Encounter chance computation & roll
        let mut trigger_enc = false;
        if !self.encounter_occurred_today {
            if let Some(rng) = self.rng.as_mut() {
                let roll: f32 = rng.random();
                if roll < self.encounter_chance_today {
                    trigger_enc = true;
                }
            }
        }
        if trigger_enc
            && let (Some(rng), Some(data)) = (self.rng.as_mut(), self.data.as_ref())
            && let Some(enc) = pick_encounter(data, self.mode.is_deep(), self.region, rng)
        {
            self.current_encounter = Some(enc.clone());
            self.encounter_occurred_today = true;
            return (false, String::from("log.encounter"), breakdown_started);
        }

        // Step 5: Executive Orders
        let idx = ((self.day.saturating_sub(1)) / 4) as usize % ExecOrder::ALL.len();
        self.current_order = ExecOrder::ALL[idx];
        let mut supplies_cost = 1;
        let mut sanity_cost = 1;
        self.current_order
            .apply_daily(self.day, &mut supplies_cost, &mut sanity_cost);
        self.stats.supplies -= supplies_cost;
        self.stats.sanity -= sanity_cost;
        self.stats.pants += 1;
        self.stats.clamp();

        // Step 6: Distance/region update and day advance
        self.day += 1;
        self.region = Self::region_by_day(self.day);
        self.encounter_occurred_today = false;

        // Check for failure conditions after all effects
        if self.stats.pants >= 100 || self.stats.hp <= 0 || self.stats.sanity <= 0 {
            return (true, String::from("log.pants-emergency"), breakdown_started);
        }

        // If travel is blocked by breakdown, don't continue
        if self.travel_blocked {
            return (false, String::from("log.travel-blocked"), breakdown_started);
        }

        (false, String::from("log.traveled"), breakdown_started)
    }

    /// Apply weather effects as step 2 of daily tick
    fn apply_weather_effects(&mut self) {
        if let Some(weather_config) = self.data.as_ref().and_then(|d| {
            serde_json::from_str::<WeatherConfig>(&serde_json::to_string(d).unwrap_or_default())
                .ok()
        }) {
            crate::weather::apply_weather_effects(self, &weather_config);
        }
    }

    /// Apply vehicle breakdown logic
    fn vehicle_roll(&mut self) -> bool {
        let mut breakdown_started = false;
        if self.rng.is_some() && self.breakdown.is_none() {
            let breakdown_chance = 0.1; // 10% chance per day
            let roll: f32 = self.rng.as_mut().unwrap().random();
            if roll < breakdown_chance {
                use crate::vehicle::Part;
                let parts = [Part::Tire, Part::Battery, Part::Alternator, Part::FuelPump];
                let part_idx: usize = self.rng.as_mut().unwrap().random_range(0..parts.len());
                self.breakdown = Some(crate::vehicle::Breakdown {
                    part: parts[part_idx],
                    day_started: i32::try_from(self.day).unwrap_or(0),
                });
                self.travel_blocked = true;
                breakdown_started = true;
            }
        }
        breakdown_started
    }

    pub fn apply_choice(&mut self, idx: usize) {
        if let Some(enc) = self.current_encounter.clone()
            && let Some(choice) = enc.choices.get(idx)
        {
            let eff = &choice.effects;
            self.stats.hp += eff.hp;
            self.stats.sanity += eff.sanity;
            self.stats.credibility += eff.credibility;
            self.stats.supplies += eff.supplies;
            self.stats.morale += eff.morale;
            self.stats.allies += eff.allies;
            self.stats.pants += eff.pants;
            if let Some(r) = &eff.add_receipt {
                self.receipts.push(r.clone());
            }
            if eff.use_receipt {
                let _ = self.receipts.pop();
            }
            if let Some(log) = &eff.log {
                self.logs.push(log.clone());
            }
        }
        self.current_encounter = None;
    }

    fn resolve_breakdown(&mut self) {
        if let Some(breakdown) = self.breakdown.clone() {
            if self.consume_spare_for_part(breakdown.part) {
                self.breakdown = None;
                self.travel_blocked = false;
                self.logs.push(String::from("log.breakdown-repaired"));
                return;
            }

            let day_started = u32::try_from(breakdown.day_started).unwrap_or(0);
            if self.day.saturating_sub(day_started) >= 1 {
                self.breakdown = None;
                self.travel_blocked = false;
                self.logs.push(String::from("log.breakdown-jury-rigged"));
            } else {
                self.travel_blocked = true;
            }
        } else {
            self.travel_blocked = false;
        }
    }

    fn consume_spare_for_part(&mut self, part: Part) -> bool {
        let spares = &mut self.inventory.spares;
        match part {
            Part::Tire if spares.tire > 0 => {
                spares.tire -= 1;
                true
            }
            Part::Battery if spares.battery > 0 => {
                spares.battery -= 1;
                true
            }
            Part::Alternator if spares.alt > 0 => {
                spares.alt -= 1;
                true
            }
            Part::FuelPump if spares.pump > 0 => {
                spares.pump -= 1;
                true
            }
            _ => false,
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        if let Some(rng) = self.rng.as_mut() {
            rng.random()
        } else {
            0
        }
    }

    pub fn next_pct(&mut self) -> u8 {
        (self.next_u32() % 100) as u8
    }

    /// Clamp all stats to valid ranges
    pub fn clamp_stats(&mut self) {
        self.stats.clamp();
    }

    /// Apply pace and diet configuration (placeholder)
    pub fn apply_pace_and_diet(&mut self, _cfg: &crate::pacing::PacingConfig) {
        // Placeholder implementation
    }

    /// Save game state (placeholder - platform specific)
    pub fn save(&self) {
        // Placeholder - web implementation will handle this
    }

    /// Load game state (placeholder - platform specific)
    #[must_use]
    pub fn load() -> Option<Self> {
        // Placeholder - web implementation will handle this
        None
    }

    /// Apply persona effects (placeholder)
    pub fn apply_persona(&mut self, persona: &Persona) {
        self.persona_id = Some(persona.id.clone());
        self.score_mult = persona.score_mult;
        self.mods = persona.mods.clone();

        if persona.start.supplies > 0 {
            self.stats.supplies = persona.start.supplies;
        }
        if persona.start.credibility > 0 {
            self.stats.credibility = persona.start.credibility;
        }
        if persona.start.sanity > 0 {
            self.stats.sanity = persona.start.sanity;
        }
        if persona.start.morale > 0 {
            self.stats.morale = persona.start.morale;
        }
        if persona.start.allies > 0 {
            self.stats.allies = persona.start.allies;
        }

        if persona.start.budget > 0 {
            self.budget = persona.start.budget;
            self.budget_cents = i64::from(persona.start.budget) * 100;
        }

        self.stats.clamp();
        self.logs
            .push(format!("log.persona.selected.{}", persona.id));
    }

    pub fn set_party<I, S>(&mut self, leader: S, companions: I)
    where
        I: IntoIterator,
        I::Item: Into<String>,
        S: Into<String>,
    {
        self.party.leader = leader.into();
        self.party.companions = companions.into_iter().map(Into::into).take(4).collect();
        while self.party.companions.len() < 4 {
            let idx = self.party.companions.len() + 2;
            self.party.companions.push(format!("Traveler {idx}"));
        }
        self.logs.push(String::from("log.party.updated"));
    }

    /// Apply store purchase effects
    pub fn apply_store_purchase(
        &mut self,
        cost_cents: i64,
        grants: &crate::store::Grants,
        tags: &[String],
    ) {
        // Subtract cost from budget
        self.budget_cents = (self.budget_cents - cost_cents).max(0);

        // Apply grants
        self.stats.supplies += grants.supplies;
        self.stats.credibility += grants.credibility;
        self.inventory.spares.tire += grants.spare_tire;
        self.inventory.spares.battery += grants.spare_battery;
        self.inventory.spares.alt += grants.spare_alt;
        self.inventory.spares.pump += grants.spare_pump;

        // Add tags
        for tag in tags {
            self.inventory.tags.insert(tag.clone());
        }

        // Clamp stats to valid ranges
        self.clamp_stats();
    }
}
