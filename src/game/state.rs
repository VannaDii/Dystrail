use gloo::storage::{LocalStorage, Storage};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::game::data::{Encounter, EncounterData};
use crate::game::encounters::pick_encounter;
use crate::game::exec_orders::ExecOrder;
use crate::game::personas::{Persona, PersonaMods};
use crate::game::vehicle::{Vehicle, Breakdown};
use crate::game::weather::{WeatherState, WeatherConfig};
use crate::game::camp::CampState;

/// Default pace setting
fn default_pace() -> String {
    "steady".to_string()
}

/// Default diet setting
fn default_diet() -> String {
    "mixed".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            pace: default_pace(),
            diet: default_diet(),
            receipt_bonus_pct: 0,
            encounter_chance_today: 0.35,
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

    pub fn save(&self) {
        let _ = LocalStorage::set("dystrail.save", self);
    }
    #[must_use]
    pub fn load() -> Option<Self> {
        LocalStorage::get("dystrail.save").ok()
    }
    #[must_use]
    pub fn region_by_day(day: u32) -> Region {
        match day {
            0..=4 => Region::Heartland,
            5..=9 => Region::RustBelt,
            _ => Region::Beltway,
        }
    }

    pub fn travel_next_leg(&mut self) -> (bool, String) {
        // Step 1: Apply Pace & Diet deltas (called from app.rs before this function)
        // (This is already handled in app.rs via apply_pace_and_diet call)

        // Step 2: Apply Weather effects
        self.apply_weather_effects();

        // Step 3: Vehicle breakdown roll
        self.step3_vehicle();

        // Step 4: Encounter chance computation & roll (now uses weather-modified encounter chance)
        let mut trigger_enc = false;
        if let Some(rng) = self.rng.as_mut() {
            let roll: f32 = rng.random();
            if roll < self.encounter_chance_today {
                trigger_enc = true;
            }
        }
        if trigger_enc
            && let (Some(rng), Some(data)) = (self.rng.as_mut(), self.data.as_ref())
            && let Some(enc) = pick_encounter(data, self.mode.is_deep(), self.region, rng)
        {
            self.current_encounter = Some(enc.clone());
            return (false, String::from("log.encounter"));
        }

        // Step 5: Executive Orders (moved after encounter roll as per spec)
        let idx = ((self.day.saturating_sub(1)) / 4) as usize % ExecOrder::ALL.len();
        self.current_order = ExecOrder::ALL[idx];
        let mut supplies_cost = 1;
        let mut sanity_cost = 1;
        self.current_order
            .apply_daily(self.day, &mut supplies_cost, &mut sanity_cost);
        self.stats.supplies -= supplies_cost;
        self.stats.sanity -= sanity_cost;
        self.stats.pants += 1;

        // Step 6: Distance/region update and day advance
        self.day += 1;
        self.region = Self::region_by_day(self.day);

        // Check for failure conditions after all effects
        if self.stats.pants >= 100 || self.stats.hp <= 0 || self.stats.sanity <= 0 {
            return (true, String::from("log.pants-emergency"));
        }

        // If travel is blocked by breakdown, don't continue
        if self.travel_blocked {
            return (false, String::from("log.travel-blocked"));
        }

        (false, String::from("log.traveled"))
    }

    /// Apply weather effects as step 2 of daily tick
    fn apply_weather_effects(&mut self) {
        // For now, use default weather config (future: load from static)
        let weather_cfg = WeatherConfig::default_config();

        // Process daily weather
        crate::game::weather::process_daily_weather(self, &weather_cfg);
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

    pub fn next_u32(&mut self) -> u32 {
        if let Some(rng) = self.rng.as_mut() {
            let v: u32 = rng.random();
            v
        } else {
            0
        }
    }
    pub fn next_pct(&mut self) -> u8 {
        (self.next_u32() % 100) as u8
    }
}

impl GameState {
    pub fn apply_persona(&mut self, p: &Persona) {
        self.persona_id = Some(p.id.clone());
        // Override starting stats (do not touch hp/pants)
        self.stats.supplies = p.start.supplies;
        self.stats.credibility = p.start.credibility;
        self.stats.sanity = p.start.sanity;
        self.stats.morale = p.start.morale;
        self.stats.allies = p.start.allies;
        self.budget = p.start.budget;
        self.budget_cents = i64::from(p.start.budget * 100); // Convert dollars to cents
        self.score_mult = p.score_mult;
        self.mods = p.mods.clone();
        self.stats.clamp();
        self.save();
    }

    /// Apply store purchases to the game state
    pub fn apply_store_purchase(&mut self, total_cost_cents: i64, grants: &crate::game::store::Grants, tags: &[String]) {
        // Deduct cost from budget
        self.budget_cents -= total_cost_cents;
        #[allow(clippy::cast_possible_truncation)]
        {
            self.budget = (self.budget_cents / 100) as i32; // Update legacy budget field
        }

        // Apply grants to stats
        self.stats.supplies += grants.supplies;
        self.stats.credibility += grants.credibility;

        // Apply grants to spares
        self.inventory.spares.tire += grants.spare_tire;
        self.inventory.spares.battery += grants.spare_battery;
        self.inventory.spares.alt += grants.spare_alt;
        self.inventory.spares.pump += grants.spare_pump;

        // Add tags (using set semantics - no duplicates)
        for tag in tags {
            self.inventory.tags.insert(tag.clone());
        }

        // Clamp stats to valid ranges
        self.stats.clamp();

        // Save the updated state
        self.save();
    }

    /// Check if the player has enough budget for a purchase
    #[must_use]
    pub fn can_afford(&self, cost_cents: i64) -> bool {
        self.budget_cents >= cost_cents
    }

    /// Get the remaining budget in cents
    #[must_use]
    pub fn remaining_budget_cents(&self) -> i64 {
        self.budget_cents
    }

    /// Apply pace and diet settings to game state for the current day
    pub fn apply_pace_and_diet(&mut self, config: &crate::game::pacing::PacingConfig) {
        let pace_cfg = config.get_pace_safe(&self.pace);
        let diet_cfg = config.get_diet_safe(&self.diet);

        // Apply sanity and pants deltas
        self.stats.sanity += pace_cfg.sanity + diet_cfg.sanity;
        self.stats.pants = (self.stats.pants + pace_cfg.pants + diet_cfg.pants)
            .clamp(config.limits.pants_floor, config.limits.pants_ceiling);

        // Calculate receipt bonus (used by forage/receipt-finding events)
        // Note: persona bonus would be added here if personas provide receipt bonuses
        self.receipt_bonus_pct = diet_cfg.receipt_find_pct_delta;

        // Set encounter chance base (pace delta applied to base)
        let base = config.limits.encounter_base + pace_cfg.encounter_chance_delta;
        self.encounter_chance_today = base.clamp(0.0, 1.0);

        // Set distance multiplier for today
        self.distance_today = config.limits.distance_base * pace_cfg.dist_mult;

        // Clamp stats to valid ranges
        self.stats.clamp();
    }

    /// Step 3 of daily tick: Vehicle breakdown roll and status check
    pub fn step3_vehicle(&mut self) {
        // If breakdown already exists, travel is blocked
        if self.breakdown.is_some() {
            self.travel_blocked = true;
            return;
        }

        // Load vehicle config (for now use defaults, future: async load)
        let cfg = crate::game::vehicle::VehicleConfig::default();

        // Perform breakdown roll with current weather
        if let Some(rng) = self.rng.as_mut() {
            let weather_str = match self.weather_state.today {
                crate::game::weather::Weather::Clear => "Clear",
                crate::game::weather::Weather::Storm => "Storm",
                crate::game::weather::Weather::HeatWave => "HeatWave",
                crate::game::weather::Weather::ColdSnap => "ColdSnap",
                crate::game::weather::Weather::Smoke => "Smoke",
            };

            if let Some(part) = crate::game::vehicle::breakdown_roll(&self.pace, weather_str, &cfg, rng) {
                #[allow(clippy::cast_possible_wrap)]
                let day_started = self.day as i32;
                self.breakdown = Some(crate::game::vehicle::Breakdown {
                    part,
                    day_started,
                });
                self.travel_blocked = true;
                // Log the breakdown (UI will announce via aria-live)
                self.logs.push(format!("vehicle.breakdown.{part:?}").to_lowercase());
            }
        }
    }

    /// Try to use a spare part to fix the current breakdown
    pub fn try_use_spare(&mut self) -> bool {
        if let Some(breakdown) = &self.breakdown {
            let cfg = crate::game::vehicle::VehicleConfig::default();
            let part = breakdown.part;
            let available = match part {
                crate::game::vehicle::Part::Tire => self.inventory.spares.tire > 0,
                crate::game::vehicle::Part::Battery => self.inventory.spares.battery > 0,
                crate::game::vehicle::Part::Alternator => self.inventory.spares.alt > 0,
                crate::game::vehicle::Part::FuelPump => self.inventory.spares.pump > 0,
            };

            if available {
                // Consume the spare
                match part {
                    crate::game::vehicle::Part::Tire => self.inventory.spares.tire -= 1,
                    crate::game::vehicle::Part::Battery => self.inventory.spares.battery -= 1,
                    crate::game::vehicle::Part::Alternator => self.inventory.spares.alt -= 1,
                    crate::game::vehicle::Part::FuelPump => self.inventory.spares.pump -= 1,
                }

                // Apply repair costs
                self.stats.supplies -= cfg.repair_costs.use_spare_supplies;

                // Clear breakdown
                self.breakdown = None;
                self.travel_blocked = false;

                // Log repair
                self.logs.push(format!("vehicle.repair.spare.{part:?}").to_lowercase());
                return true;
            }
        }
        false
    }

    /// Apply hack fix to clear breakdown
    pub fn hack_fix(&mut self) {
        if self.breakdown.is_some() {
            let cfg = crate::game::vehicle::VehicleConfig::default();

            // Apply costs
            self.stats.supplies -= cfg.repair_costs.hack_supplies;
            self.stats.credibility -= cfg.repair_costs.hack_cred;
            #[allow(clippy::cast_sign_loss)]
            {
                self.day += cfg.repair_costs.hack_day as u32;
            }

            // Clear breakdown
            self.breakdown = None;
            self.travel_blocked = false;

            // Log hack fix
            self.logs.push("vehicle.repair.hack".to_string());
        }
    }

    /// Wait for mechanic (future hook for encounters)
    pub fn wait_mechanic(&mut self) {
        let cfg = crate::game::vehicle::VehicleConfig::default();

        // Always advance day
        #[allow(clippy::cast_sign_loss)]
        {
            self.day += cfg.mechanic_hook.day_cost as u32;
        }

        // If mechanic hook is enabled, chance to clear breakdown
        if cfg.mechanic_hook.enabled
            && self.breakdown.is_some()
            && let Some(rng) = self.rng.as_mut()
        {
            let roll: f32 = rng.random();
            if roll < cfg.mechanic_hook.chance_clear {
                self.breakdown = None;
                self.travel_blocked = false;
                self.logs.push("vehicle.repair.mechanic".to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::pacing::PacingConfig;

    #[test]
    fn test_default_game_state_has_correct_pace_diet() {
        let game_state = GameState::default();

        assert_eq!(game_state.pace, "steady");
        assert_eq!(game_state.diet, "mixed");
        assert!((game_state.encounter_chance_today - 0.35).abs() < f32::EPSILON); // default base
        assert!((game_state.distance_today - 1.0).abs() < f32::EPSILON); // default distance
        assert_eq!(game_state.receipt_bonus_pct, 0);
        assert_eq!(game_state.day, 1);
        assert_eq!(game_state.region, Region::Heartland);
        assert_eq!(game_state.mode, GameMode::Classic);
    }

    #[test]
    fn test_apply_pace_and_diet_steady_mixed() {
        let config = PacingConfig::default_config();
        let mut game_state = GameState::default();

        let initial_sanity = game_state.stats.sanity;
        let initial_pants = game_state.stats.pants;

        // Apply steady pace and mixed diet (should be neutral)
        game_state.apply_pace_and_diet(&config);

        assert_eq!(game_state.stats.sanity, initial_sanity);
        assert_eq!(game_state.stats.pants, initial_pants);
        assert!((game_state.encounter_chance_today - 0.35).abs() < f32::EPSILON); // base = 0.35 + 0.0 delta
        assert_eq!(game_state.receipt_bonus_pct, 0); // mixed base
        assert!(game_state.distance_today > 0.0);
    }

    #[test]
    fn test_apply_pace_and_diet_with_effects() {
        let config = PacingConfig::default_config();
        let mut game_state = GameState {
            pace: "heated".to_string(), // sanity: -1, pants: +3
            diet: "doom".to_string(), // sanity: -2, pants: +4
            ..Default::default()
        };

        let initial_sanity = game_state.stats.sanity;
        let initial_pants = game_state.stats.pants;

        game_state.apply_pace_and_diet(&config);

        // Should have lower sanity (-3 total) and higher pants (+7 total)
        assert_eq!(game_state.stats.sanity, initial_sanity - 3);
        assert_eq!(game_state.stats.pants, initial_pants + 7);
        assert!((game_state.encounter_chance_today - 0.4).abs() < f32::EPSILON); // 0.35 base + 0.05 delta
        assert_eq!(game_state.receipt_bonus_pct, 8); // doom receipt bonus
    }

    #[test]
    fn test_apply_pace_and_diet_stats_clamping() {
        let config = PacingConfig::default_config();
        let mut game_state = GameState::default();

        // Set stats to extreme values
        game_state.stats.sanity = 0;
        game_state.stats.pants = 95;

        // Apply effects that would push beyond limits
        game_state.pace = "blitz".to_string();
        game_state.diet = "doom".to_string();

        game_state.apply_pace_and_diet(&config);

        // Stats should be clamped to valid ranges
        assert!(game_state.stats.sanity >= 0);
        assert!(game_state.stats.sanity <= 100);
        assert!(game_state.stats.pants >= 0);
        assert!(game_state.stats.pants <= 100);
    }

    #[test]
    fn test_apply_pace_and_diet_invalid_options() {
        let config = PacingConfig::default_config();
        let mut game_state = GameState {
            pace: "invalid_pace".to_string(),
            diet: "invalid_diet".to_string(),
            ..Default::default()
        };

        let initial_sanity = game_state.stats.sanity;
        let initial_pants = game_state.stats.pants;

        // Should fall back to defaults without crashing
        game_state.apply_pace_and_diet(&config);

        // Should use defaults (steady/mixed)
        assert_eq!(game_state.stats.sanity, initial_sanity);
        assert_eq!(game_state.stats.pants, initial_pants);
        assert!((game_state.encounter_chance_today - 0.35).abs() < f32::EPSILON); // base encounter chance
        assert_eq!(game_state.receipt_bonus_pct, 0);
    }

    #[test]
    fn test_apply_pace_and_diet_distance_calculation() {
        let config = PacingConfig::default_config();
        let mut game_state = GameState {
            pace: "steady".to_string(),
            ..Default::default()
        };

        // Test different paces
        game_state.apply_pace_and_diet(&config);
        let steady_distance = game_state.distance_today; // 1.0 * 1.0

        game_state.pace = "heated".to_string();
        game_state.apply_pace_and_diet(&config);
        let heated_distance = game_state.distance_today; // 1.0 * 1.2

        game_state.pace = "blitz".to_string();
        game_state.apply_pace_and_diet(&config);
        let blitz_distance = game_state.distance_today; // 1.0 * 1.4

        // Higher pace should yield more distance
        assert!(heated_distance > steady_distance);
        assert!(blitz_distance > heated_distance);

        // Check specific expected values
        assert!((steady_distance - 1.0).abs() < f32::EPSILON);
        assert!((heated_distance - 1.2).abs() < f32::EPSILON);
        assert!((blitz_distance - 1.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_pace_and_diet_encounter_chance() {
        let config = PacingConfig::default_config();
        let mut game_state = GameState {
            pace: "steady".to_string(),
            ..GameState::default()
        };

        // Test different paces for encounter chance
        game_state.apply_pace_and_diet(&config);
        let steady_chance = game_state.encounter_chance_today; // 0.35 + 0.0

        game_state.pace = "heated".to_string();
        game_state.apply_pace_and_diet(&config);
        let heated_chance = game_state.encounter_chance_today; // 0.35 + 0.05

        game_state.pace = "blitz".to_string();
        game_state.apply_pace_and_diet(&config);
        let blitz_chance = game_state.encounter_chance_today; // 0.35 + 0.1

        // Higher pace should yield higher encounter chance
        assert!(heated_chance > steady_chance);
        assert!(blitz_chance > heated_chance);

        // Check specific expected values
        assert!((steady_chance - 0.35).abs() < f32::EPSILON);
        assert!((heated_chance - 0.4).abs() < f32::EPSILON);
        assert!((blitz_chance - 0.45).abs() < f32::EPSILON);

        // All chances should be within valid range
        assert!((0.0..=1.0).contains(&steady_chance));
        assert!((0.0..=1.0).contains(&heated_chance));
        assert!((0.0..=1.0).contains(&blitz_chance));
    }

    #[test]
    fn test_vehicle_spare_usage() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;

        // Create test GameState with breakdown
        let mut gs = GameState {
            rng: Some(ChaCha20Rng::seed_from_u64(42)),
            breakdown: Some(crate::game::vehicle::Breakdown {
                part: crate::game::vehicle::Part::Tire,
                day_started: 1,
            }),
            travel_blocked: true,
            ..GameState::default()
        };

        // Test using spare when available
        gs.inventory.spares.tire = 2;
        gs.stats.supplies = 10;

        let success = gs.try_use_spare();
        assert!(success, "Should successfully use spare tire");
        assert_eq!(gs.inventory.spares.tire, 1, "Should consume one spare tire");
        assert_eq!(gs.stats.supplies, 9, "Should consume 1 supply");
        assert!(gs.breakdown.is_none(), "Should clear breakdown");
        assert!(!gs.travel_blocked, "Should unblock travel");
    }

    #[test]
    fn test_vehicle_spare_usage_no_spare() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;

        let mut gs = GameState {
            rng: Some(ChaCha20Rng::seed_from_u64(42)),
            breakdown: Some(crate::game::vehicle::Breakdown {
                part: crate::game::vehicle::Part::Battery,
                day_started: 1,
            }),
            travel_blocked: true,
            ..GameState::default()
        };

        // Set up no spares
        gs.inventory.spares.battery = 0;

        let success = gs.try_use_spare();
        assert!(!success, "Should fail when no spare available");
        assert!(gs.breakdown.is_some(), "Should not clear breakdown");
        assert!(gs.travel_blocked, "Should remain blocked");
    }

    #[test]
    fn test_vehicle_hack_fix() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;

        let mut gs = GameState {
            rng: Some(ChaCha20Rng::seed_from_u64(42)),
            breakdown: Some(crate::game::vehicle::Breakdown {
                part: crate::game::vehicle::Part::Alternator,
                day_started: 5,
            }),
            travel_blocked: true,
            day: 5,
            ..GameState::default()
        };

        gs.stats.supplies = 10;
        gs.stats.credibility = 10;

        gs.hack_fix();

        assert!(gs.breakdown.is_none(), "Should clear breakdown");
        assert!(!gs.travel_blocked, "Should unblock travel");
        assert_eq!(gs.stats.supplies, 7, "Should consume 3 supplies");
        assert_eq!(gs.stats.credibility, 9, "Should lose 1 credibility");
        assert_eq!(gs.day, 6, "Should advance 1 day");
    }

    #[test]
    fn test_vehicle_wait_mechanic() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;

        let mut gs = GameState {
            rng: Some(ChaCha20Rng::seed_from_u64(42)),
            breakdown: Some(crate::game::vehicle::Breakdown {
                part: crate::game::vehicle::Part::FuelPump,
                day_started: 3,
            }),
            day: 3,
            ..GameState::default()
        };

        gs.wait_mechanic();

        assert_eq!(gs.day, 4, "Should advance 1 day");
        // Breakdown should remain (wait_mechanic is basic implementation)
        assert!(gs.breakdown.is_some(), "Breakdown should remain in basic implementation");
    }

    #[test]
    fn test_vehicle_step3_integration() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;

        let mut gs = GameState {
            rng: Some(ChaCha20Rng::seed_from_u64(42)),
            breakdown: Some(crate::game::vehicle::Breakdown {
                part: crate::game::vehicle::Part::Tire,
                day_started: 1,
            }),
            ..GameState::default()
        };

        // Test that existing breakdown blocks travel
        gs.step3_vehicle();

        assert!(gs.travel_blocked, "Should remain blocked with existing breakdown");

        // Test with no breakdown - should potentially create one (probabilistic)
        gs.breakdown = None;
        gs.travel_blocked = false;

        // For deterministic testing, we'll just test that the function doesn't panic
        // and handles the no-breakdown case properly
        gs.step3_vehicle();
        // The function should complete without error regardless of breakdown outcome
    }
}
