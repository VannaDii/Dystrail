use gloo::storage::{LocalStorage, Storage};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

use crate::game::data::{Encounter, EncounterData};
use crate::game::encounters::pick_encounter;
use crate::game::exec_orders::ExecOrder;
use crate::game::personas::{Persona, PersonaMods};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(default)]
    pub persona_id: Option<String>,
    #[serde(default)]
    pub score_mult: f32,
    #[serde(default)]
    pub mods: PersonaMods,
    pub logs: Vec<String>,
    pub receipts: Vec<String>,
    pub current_encounter: Option<Encounter>,
    pub current_order: ExecOrder,
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
            persona_id: None,
            score_mult: 1.0,
            mods: PersonaMods::default(),
            logs: vec![String::from("log.booting")],
            receipts: vec![],
            current_encounter: None,
            current_order: ExecOrder::Shutdown,
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
        let mut supplies_cost = 1;
        let mut sanity_cost = 1;
        // rotate EO every 4 days
        let idx = ((self.day.saturating_sub(1)) / 4) as usize % ExecOrder::ALL.len();
        self.current_order = ExecOrder::ALL[idx];
        self.current_order
            .apply_daily(self.day, &mut supplies_cost, &mut sanity_cost);
        self.stats.supplies -= supplies_cost;
        self.stats.sanity -= sanity_cost;
        self.stats.pants += 1;
        self.day += 1;
        self.region = Self::region_by_day(self.day);
        if self.stats.pants >= 100 || self.stats.hp <= 0 || self.stats.sanity <= 0 {
            return (true, String::from("log.pants-emergency"));
        }
        let mut trigger_enc = false;
        if let Some(rng) = self.rng.as_mut() {
            let roll: f32 = rng.random();
            if roll < 0.35 {
                trigger_enc = true;
            }
        }
        if trigger_enc {
            if let (Some(rng), Some(data)) = (self.rng.as_mut(), self.data.as_ref()) {
                if let Some(enc) = pick_encounter(data, self.mode.is_deep(), self.region, rng) {
                    self.current_encounter = Some(enc.clone());
                    return (false, String::from("log.encounter"));
                }
            }
        }
        (false, String::from("log.traveled"))
    }

    pub fn apply_choice(&mut self, idx: usize) {
        if let Some(enc) = self.current_encounter.clone() {
            if let Some(choice) = enc.choices.get(idx) {
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
        self.score_mult = p.score_mult;
        self.mods = p.mods.clone();
        self.stats.clamp();
        self.save();
    }
}
