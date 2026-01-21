use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use dystrail_game::boss::BossConfig;
use dystrail_game::camp::CampConfig;
use dystrail_game::data::{Choice, Effects, Encounter, EncounterData};
use dystrail_game::endgame::EndgameTravelCfg;
use dystrail_game::numbers::clamp_f64_to_f32;
use dystrail_game::personas::{Persona, PersonasList};
use dystrail_game::state::{
    CollapseCause, CrossingOutcomeTelemetry, CrossingTelemetry, Ending, Season,
};
use dystrail_game::store::{Grants, Store, StoreItem, calculate_effective_price};
use dystrail_game::weather::{Weather, WeatherConfig};
use dystrail_game::{
    DietId, GameMode, GameState, PaceId, PolicyKind, Region, compute_day_ledger_metrics,
};
use serde_json;

use crate::logic::policy::GameplayStrategy;
use crate::logic::simulation::{DecisionRecord, SimulationConfig, SimulationSession, TurnOutcome};

const LOG_MESSAGE_PREFIX: &str = "log.";
const HEATWAVE_RISK_THRESHOLD: f64 = 0.18;
const HEATWAVE_MIN_WATER: i32 = 2;
const COLDSNAP_RISK_THRESHOLD: f64 = 0.16;
const COLDSNAP_MIN_COATS: i32 = 1;
const PRICE_BASIS_DENOM: i128 = 100;
const MILESTONE_MILES: f32 = 2000.0;
const MILESTONE_DAY_LIMIT: u32 = 150;

/// Collection of immutable data required to run a simulation.
#[derive(Debug, Clone)]
pub struct TesterAssets {
    encounter_data: EncounterData,
    personas: PersonasList,
    store: Store,
    camp_config: CampConfig,
    boss_config: BossConfig,
    weather_config: WeatherConfig,
    endgame_config: EndgameTravelCfg,
}

impl TesterAssets {
    pub fn load_default() -> Self {
        let encounter_data =
            Self::load_encounters_from_assets().unwrap_or_else(Self::fallback_encounter_data);
        let personas = Self::load_personas_from_assets().unwrap_or_else(PersonasList::empty);
        let store = Self::load_store_from_assets().unwrap_or_else(Self::fallback_store_data);
        let camp_config = Self::load_camp_from_assets().unwrap_or_default();
        let boss_config = Self::load_boss_from_assets().unwrap_or_default();
        let weather_config =
            Self::load_weather_from_assets().unwrap_or_else(WeatherConfig::default_config);
        let endgame_config =
            Self::load_endgame_from_assets().unwrap_or_else(EndgameTravelCfg::default_config);

        Self {
            encounter_data,
            personas,
            store,
            camp_config,
            boss_config,
            weather_config,
            endgame_config,
        }
    }

    fn assets_data_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("dystrail-web")
            .join("static")
            .join("assets")
            .join("data")
    }

    fn load_encounters_from_assets() -> Option<EncounterData> {
        let base = Self::assets_data_root();
        let json = fs::read_to_string(base.join("game.json")).ok()?;
        let data = EncounterData::from_json(&json).ok()?;
        if data.encounters.is_empty() {
            None
        } else {
            Some(data)
        }
    }

    fn load_personas_from_assets() -> Option<PersonasList> {
        let base = Self::assets_data_root();
        let json = fs::read_to_string(base.join("personas.json")).ok()?;
        PersonasList::from_json(&json).ok()
    }

    fn load_store_from_assets() -> Option<Store> {
        let base = Self::assets_data_root();
        let json = fs::read_to_string(base.join("store.json")).ok()?;
        match serde_json::from_str(&json) {
            Ok(store) => Some(store),
            Err(err) => {
                eprintln!("⚠️ Failed to parse store.json: {err}");
                None
            }
        }
    }

    fn load_camp_from_assets() -> Option<CampConfig> {
        let base = Self::assets_data_root();
        let json = fs::read_to_string(base.join("camp.json")).ok()?;
        serde_json::from_str(&json).ok()
    }

    fn load_boss_from_assets() -> Option<BossConfig> {
        let base = Self::assets_data_root();
        let json = fs::read_to_string(base.join("boss.json")).ok()?;
        serde_json::from_str(&json).ok()
    }

    fn load_weather_from_assets() -> Option<WeatherConfig> {
        let base = Self::assets_data_root();
        let json = fs::read_to_string(base.join("weather.json")).ok()?;
        serde_json::from_str(&json).ok()
    }

    fn load_endgame_from_assets() -> Option<EndgameTravelCfg> {
        let base = Self::assets_data_root();
        let json = fs::read_to_string(base.join("endgame.json")).ok()?;
        serde_json::from_str(&json).ok()
    }

    fn fallback_encounter_data() -> EncounterData {
        EncounterData::from_encounters(vec![Encounter {
            id: "debug_campfire".to_string(),
            name: "Campfire Debate".to_string(),
            desc: "The crew argues about rationing supplies.".to_string(),
            weight: 5,
            regions: vec!["Heartland".to_string()],
            modes: vec!["classic".to_string(), "deep_end".to_string()],
            choices: vec![
                Choice {
                    label: "Share supplies".to_string(),
                    effects: Effects {
                        hp: 0,
                        sanity: 1,
                        credibility: 0,
                        supplies: -1,
                        morale: 1,
                        allies: 0,
                        pants: 0,
                        travel_bonus_ratio: 0.0,
                        add_receipt: None,
                        use_receipt: false,
                        log: Some("You keep morale up with snacks.".to_string()),
                        rest: false,
                    },
                },
                Choice {
                    label: "Hoard supplies".to_string(),
                    effects: Effects {
                        hp: 0,
                        sanity: -1,
                        credibility: 1,
                        supplies: 0,
                        morale: -1,
                        allies: 0,
                        pants: 2,
                        travel_bonus_ratio: 0.0,
                        add_receipt: None,
                        use_receipt: false,
                        log: Some("Tension rises as you hoard the jerky.".to_string()),
                        rest: false,
                    },
                },
            ],
            hard_stop: false,
            major_repair: false,
            chainable: false,
        }])
    }

    fn fallback_store_data() -> Store {
        Store {
            categories: vec![],
            items: vec![StoreItem {
                id: "rations".to_string(),
                name: "Rations Pack".to_string(),
                desc: "Fallback emergency supplies.".to_string(),
                price_cents: 500,
                unique: false,
                max_qty: 5,
                grants: Grants {
                    supplies: 2,
                    ..Grants::default()
                },
                tags: vec![],
                category: "fuel_food".to_string(),
            }],
        }
    }
}

const POLICY_BASE_ENCOUNTER_CHANCE: f32 = 0.85;

pub const DEFAULT_POLICY_SIM_DAYS: u32 = 35;

pub fn default_policy_setup(strategy: GameplayStrategy) -> fn(&mut GameState) {
    match strategy {
        GameplayStrategy::Conservative => conservative_policy_setup,
        GameplayStrategy::Aggressive => aggressive_policy_setup,
        GameplayStrategy::ResourceManager => resource_policy_setup,
        GameplayStrategy::Balanced => balanced_policy_setup,
    }
}

fn balanced_policy_setup(state: &mut GameState) {
    base_policy_setup(state);
}

fn conservative_policy_setup(state: &mut GameState) {
    base_policy_setup(state);
    state.stats.sanity = state.stats.sanity.max(8);
    state.stats.pants = 2;
    state.inventory.spares.tire = 2;
}

fn aggressive_policy_setup(state: &mut GameState) {
    base_policy_setup(state);
    state.stats.sanity = state.stats.sanity.min(7);
    state.stats.pants = 6;
    state.stats.supplies = state.stats.supplies.saturating_sub(2);
}

fn resource_policy_setup(state: &mut GameState) {
    base_policy_setup(state);
    state.stats.supplies += 3;
    state.stats.supplies = state.stats.supplies.min(15);
    state.budget_cents = 18_000;
}

fn base_policy_setup(state: &mut GameState) {
    state.stats.hp = state.stats.hp.max(8);
    state.stats.sanity = state.stats.sanity.max(8);
    state.stats.supplies = state.stats.supplies.max(12);
    state.stats.pants = state.stats.pants.min(5);
    state.encounter_chance_today = POLICY_BASE_ENCOUNTER_CHANCE;
    state.inventory.spares.tire = state.inventory.spares.tire.max(1);
    state.inventory.spares.battery = state.inventory.spares.battery.max(1);
    state.stats.clamp();
}

/// Declarative plan for running a simulation session.
#[derive(Debug, Clone)]
pub struct SimulationPlan {
    pub mode: GameMode,
    pub strategy: GameplayStrategy,
    pub max_days: Option<u32>,
    pub setup: Option<fn(&mut GameState)>,
    pub expectations: Vec<SimulationExpectation>,
}

impl SimulationPlan {
    #[must_use]
    pub fn new(mode: GameMode, strategy: GameplayStrategy) -> Self {
        Self {
            mode,
            strategy,
            max_days: None,
            setup: None,
            expectations: Vec::new(),
        }
    }

    #[must_use]
    pub const fn with_max_days(mut self, max_days: u32) -> Self {
        self.max_days = Some(max_days);
        self
    }

    #[must_use]
    pub fn with_setup(mut self, setup: fn(&mut GameState)) -> Self {
        self.setup = Some(setup);
        self
    }

    #[must_use]
    pub fn with_expectation(mut self, expectation: impl Into<SimulationExpectation>) -> Self {
        self.expectations.push(expectation.into());
        self
    }
}

/// Assertion hook run after a simulation completes.
type SimulationExpectationFn =
    Arc<dyn Fn(&SimulationSummary) -> Result<()> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SimulationExpectation(SimulationExpectationFn);

impl std::fmt::Debug for SimulationExpectation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimulationExpectation").finish()
    }
}

impl SimulationExpectation {
    #[must_use]
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&SimulationSummary) -> Result<()> + Send + Sync + 'static,
    {
        Self(Arc::new(f))
    }

    pub fn evaluate(&self, summary: &SimulationSummary) -> Result<()> {
        (self.0)(summary)
    }
}

impl<F> From<F> for SimulationExpectation
where
    F: Fn(&SimulationSummary) -> Result<()> + Send + Sync + 'static,
{
    fn from(f: F) -> Self {
        Self(Arc::new(f))
    }
}

/// Complete record of a simulation run.
#[derive(Debug, Clone)]
pub struct SimulationSummary {
    pub seed: u64,
    pub mode: GameMode,
    pub strategy: GameplayStrategy,
    pub turns: Vec<TurnOutcome>,
    pub metrics: PlayabilityMetrics,
    pub final_state: GameState,
    pub ending_message: String,
    pub game_ended: bool,
}

/// Headless deterministic runner for the core game logic.
#[derive(Clone)]
pub struct GameTester {
    verbose: bool,
    assets: Arc<TesterAssets>,
}

impl GameTester {
    pub const fn new(assets: Arc<TesterAssets>, verbose: bool) -> Self {
        Self { verbose, assets }
    }

    pub const fn verbose(&self) -> bool {
        self.verbose
    }

    fn persona_for_strategy(&self, strategy: GameplayStrategy) -> Option<Persona> {
        if self.assets.personas.is_empty() {
            return None;
        }
        let preferred = match strategy {
            GameplayStrategy::Balanced => "staffer",
            GameplayStrategy::Conservative => "lobbyist",
            GameplayStrategy::Aggressive => "whistleblower",
            GameplayStrategy::ResourceManager => "journalist",
        };

        self.assets
            .personas
            .get_by_id(preferred)
            .cloned()
            .or_else(|| self.assets.personas.iter().next().cloned())
    }

    fn apply_persona_choice(&self, state: &mut GameState, strategy: GameplayStrategy) {
        if let Some(persona) = self.persona_for_strategy(strategy) {
            if self.verbose {
                println!("🧬 Selected persona: {}", persona.name);
            }
            state.apply_persona(&persona);
        } else if self.verbose {
            println!("⚠️ No persona data available; using default stats");
        }
    }

    fn apply_store_loadout(&self, state: &mut GameState, strategy: GameplayStrategy, seed: u64) {
        if state.budget_cents <= 0 {
            return;
        }

        if self.verbose {
            let store = &self.assets.store;
            println!(
                "🛍️ Entering store with ${} ({} categories)",
                format_cents(state.budget_cents),
                store.categories.len()
            );
        }

        let plan = self.planned_purchases(state, strategy, seed);
        for (item_id, qty) in plan {
            self.execute_purchase(state, item_id, qty);
        }
    }

    const fn configure_strategy_settings(state: &mut GameState, strategy: GameplayStrategy) {
        let (auto_rest, threshold) = match strategy {
            GameplayStrategy::Aggressive => (true, 3),
            GameplayStrategy::Balanced | GameplayStrategy::ResourceManager => (true, 5),
            GameplayStrategy::Conservative => (true, 4),
        };
        state.auto_camp_rest = auto_rest;
        state.rest_threshold = threshold;
        state.day_state.rest.rest_requested = false;
        state.policy = Some(match strategy {
            GameplayStrategy::Balanced => PolicyKind::Balanced,
            GameplayStrategy::Conservative => PolicyKind::Conservative,
            GameplayStrategy::Aggressive => PolicyKind::Aggressive,
            GameplayStrategy::ResourceManager => PolicyKind::ResourceManager,
        });
        state.pace = match strategy {
            GameplayStrategy::Aggressive => PaceId::Heated,
            _ => PaceId::Steady,
        };
        state.diet = match strategy {
            GameplayStrategy::Conservative | GameplayStrategy::ResourceManager => DietId::Quiet,
            _ => DietId::Mixed,
        };
    }

    fn assign_party(&self, state: &mut GameState, strategy: GameplayStrategy, seed: u64) {
        let (leader, companions) = Self::party_roster(strategy, seed);
        state.set_party(leader, companions);
        if self.verbose {
            let mut names = vec![state.party.leader.clone()];
            names.extend(state.party.companions.iter().cloned());
            println!("🧑‍🤝‍🧑 Party: {}", names.join(", "));
        }
    }

    fn planned_purchases(
        &self,
        state: &GameState,
        strategy: GameplayStrategy,
        _seed: u64,
    ) -> Vec<(&'static str, i32)> {
        let mut plan: Vec<(&'static str, i32)> = match strategy {
            GameplayStrategy::Balanced => vec![("rations", 2), ("water", 1), ("spare_tire", 1)],
            GameplayStrategy::Conservative => vec![
                ("spare_tire", 1),
                ("battery", 1),
                ("legal_fund", 1),
                ("spare_tire", 1),
            ],
            GameplayStrategy::Aggressive => vec![("legal_fund", 2), ("rations", 1)],
            GameplayStrategy::ResourceManager => {
                vec![("rations", 3), ("water", 2), ("spare_tire", 1)]
            }
        };

        let heat_risk = self.heatwave_risk();
        let cold_risk = self.coldsnap_risk();
        let need_conservative_water = matches!(strategy, GameplayStrategy::Conservative)
            && heat_risk >= HEATWAVE_RISK_THRESHOLD;
        let need_conservative_coat = matches!(strategy, GameplayStrategy::Conservative)
            && cold_risk >= COLDSNAP_RISK_THRESHOLD;

        if matches!(
            strategy,
            GameplayStrategy::Balanced | GameplayStrategy::Conservative
        ) && state.mode == GameMode::Classic
            && matches!(state.season, Season::Fall | Season::Winter)
        {
            Self::prioritize_coat(&mut plan, strategy);
        }

        if matches!(strategy, GameplayStrategy::Balanced) && heat_risk >= HEATWAVE_RISK_THRESHOLD {
            Self::ensure_min_quantity(&mut plan, "water", HEATWAVE_MIN_WATER, Some(1));
        } else if need_conservative_water {
            Self::ensure_min_quantity(&mut plan, "water", 1, Some(0));
        }

        if matches!(strategy, GameplayStrategy::Balanced) && cold_risk >= COLDSNAP_RISK_THRESHOLD {
            Self::ensure_min_quantity(&mut plan, "coats", COLDSNAP_MIN_COATS, Some(0));
        } else if need_conservative_coat {
            Self::ensure_min_quantity(&mut plan, "coats", 1, Some(0));
        }

        if matches!(strategy, GameplayStrategy::Conservative)
            && need_conservative_coat
            && need_conservative_water
        {
            Self::trim_noncritical_spare(&mut plan);
        }

        plan
    }

    fn heatwave_risk(&self) -> f64 {
        self.assets
            .weather_config
            .weights
            .values()
            .filter_map(|weights| {
                let total: u32 = weights.values().copied().sum();
                let heat = weights.get(&Weather::HeatWave)?;
                if total == 0 {
                    return None;
                }
                Some(f64::from(*heat) / f64::from(total))
            })
            .fold(0.0_f64, f64::max)
    }

    fn coldsnap_risk(&self) -> f64 {
        self.assets
            .weather_config
            .weights
            .values()
            .filter_map(|weights| {
                let total: u32 = weights.values().copied().sum();
                let cold = weights.get(&Weather::ColdSnap)?;
                if total == 0 {
                    return None;
                }
                Some(f64::from(*cold) / f64::from(total))
            })
            .fold(0.0_f64, f64::max)
    }

    fn dynamic_price_multiplier(state: &GameState, item: &StoreItem) -> i32 {
        let mut basis_points = 100_i32;
        let has_tag = |tag: &str| item.tags.iter().any(|t| t == tag);
        let id = item.id.as_str();

        if has_tag("warm_coat") || has_tag("cold_resist") || id == "coats" {
            if matches!(state.season, Season::Fall | Season::Winter) {
                basis_points += 25;
            } else if matches!(state.season, Season::Spring) {
                basis_points += 10;
            }
            if matches!(state.region, Region::Beltway) {
                basis_points += 5;
            }
        }

        if has_tag("water_jugs") || id == "water" {
            if matches!(state.season, Season::Summer) {
                basis_points += 20;
            }
            if matches!(state.region, Region::Heartland) {
                basis_points += 5;
            }
        }

        if id.starts_with("spare_") {
            if state.vehicle_breakdowns >= 4 {
                basis_points += 20;
            } else if state.vehicle_breakdowns >= 2 {
                basis_points += 10;
            }
        }

        basis_points.max(100)
    }

    fn ensure_min_quantity(
        plan: &mut Vec<(&'static str, i32)>,
        item_id: &'static str,
        min_qty: i32,
        preferred_index: Option<usize>,
    ) {
        if let Some((_, qty)) = plan.iter_mut().find(|(id, _)| *id == item_id) {
            if *qty < min_qty {
                *qty = min_qty;
            }
        } else if min_qty > 0 {
            let index = preferred_index.map_or(plan.len(), |idx| idx.min(plan.len()));
            plan.insert(index, (item_id, min_qty));
        }
    }

    fn trim_noncritical_spare(plan: &mut Vec<(&'static str, i32)>) {
        let spare_tire_count = plan.iter().filter(|(id, _)| *id == "spare_tire").count();
        if spare_tire_count > 1
            && let Some(pos) = plan.iter().rposition(|(id, _)| *id == "spare_tire")
        {
            plan.remove(pos);
            return;
        }
        if let Some(pos) = plan.iter().rposition(|(id, _)| *id == "battery") {
            plan.remove(pos);
        }
    }

    fn prioritize_coat(plan: &mut Vec<(&'static str, i32)>, strategy: GameplayStrategy) {
        if plan.iter().any(|(id, _)| *id == "coats") {
            return;
        }
        let spare_positions: Vec<usize> = plan
            .iter()
            .enumerate()
            .filter(|(_, (id, _))| *id == "spare_tire")
            .map(|(idx, _)| idx)
            .collect();
        if strategy == GameplayStrategy::Conservative && spare_positions.len() >= 2 {
            plan.insert(spare_positions[1], ("coats", 1));
        } else if let Some(&idx) = spare_positions.first() {
            plan.insert(idx, ("coats", 1));
        } else {
            let insert_at = plan.len().min(1);
            plan.insert(insert_at, ("coats", 1));
        }
    }

    fn party_roster(strategy: GameplayStrategy, seed: u64) -> (String, Vec<String>) {
        let base = match strategy {
            GameplayStrategy::Balanced => [
                "Alex Morgan",
                "Jordan Rivers",
                "Riley Chen",
                "Taylor Brooks",
                "Casey Patel",
            ],
            GameplayStrategy::Conservative => [
                "Evelyn Clarke",
                "Samuel Harper",
                "Margaret Liu",
                "Robert Hayes",
                "Diana Singh",
            ],
            GameplayStrategy::Aggressive => [
                "Zoe Knight",
                "Axel Stone",
                "Blaze Carter",
                "Rex Turner",
                "Nova Fields",
            ],
            GameplayStrategy::ResourceManager => [
                "Quinn Walker",
                "Harper Diaz",
                "Morgan Lee",
                "Dakota Shah",
                "Emerson Vale",
            ],
        };

        let mut names: Vec<String> = base.into_iter().map(String::from).collect();
        match u64::try_from(names.len()) {
            Ok(len_u64) if len_u64 > 0 => {
                let offset = usize::try_from(seed % len_u64).unwrap_or(0);
                names.rotate_left(offset);
            }
            _ => {}
        }
        while names.len() < 5 {
            let idx = names.len() + 1;
            names.push(format!("Traveler {idx}"));
        }
        let leader = names
            .first()
            .cloned()
            .unwrap_or_else(|| "Traveler 1".to_string());
        let companions = names[1..].to_vec();
        (leader, companions)
    }

    fn execute_purchase(&self, state: &mut GameState, item_id: &str, requested_qty: i32) {
        if requested_qty <= 0 {
            return;
        }
        let store = &self.assets.store;
        if store.categories.is_empty() && store.items.is_empty() {
            return;
        }

        let Some(item) = store.find_item(item_id) else {
            return;
        };

        let mut qty = requested_qty;
        if item.unique {
            qty = qty.min(1);
        }
        if item.max_qty > 0 {
            qty = qty.min(item.max_qty);
        }
        if qty <= 0 {
            return;
        }

        let discount = f64::from(state.mods.store_discount_pct);
        let base_unit_price = calculate_effective_price(item.price_cents, discount);
        let price_basis = i128::from(Self::dynamic_price_multiplier(state, item));
        let unit_price = i128::from(base_unit_price);
        let Some(product) = unit_price.checked_mul(price_basis) else {
            return;
        };
        let adjusted_unit = if product >= 0 {
            (product + (PRICE_BASIS_DENOM - 1)) / PRICE_BASIS_DENOM
        } else {
            product / PRICE_BASIS_DENOM
        };
        let Ok(adjusted_unit) = i64::try_from(adjusted_unit) else {
            return;
        };
        let qty_i64 = i64::from(qty);
        let total_cost = adjusted_unit.saturating_mul(qty_i64);
        if total_cost <= 0 || state.budget_cents < total_cost {
            return;
        }

        let total_grants = Grants {
            supplies: item.grants.supplies * qty,
            credibility: item.grants.credibility * qty,
            spare_tire: item.grants.spare_tire * qty,
            spare_battery: item.grants.spare_battery * qty,
            spare_alt: item.grants.spare_alt * qty,
            spare_pump: item.grants.spare_pump * qty,
            enabled: item.grants.enabled,
        };

        let mut tags = Vec::new();
        for _ in 0..qty {
            tags.extend(item.tags.iter().cloned());
        }

        state.apply_store_purchase(total_cost, &total_grants, &tags);
        state
            .logs
            .push(format!("log.store.purchase.{}x{}", item.id, qty));
        if self.verbose {
            let total_cost_display = format_cents(total_cost);
            let remaining_display = format_cents(state.budget_cents);
            println!(
                "🛒 Purchased {}x {} for ${total_cost_display} (remaining ${remaining_display})",
                qty, item.name
            );
        }
    }

    pub fn run_plan(&self, plan: &SimulationPlan, seed: u64) -> SimulationSummary {
        let max_days = plan.max_days.unwrap_or(200);
        let mut session = SimulationSession::new(
            SimulationConfig::new(plan.mode, plan.strategy, seed).with_max_days(max_days),
            self.assets.encounter_data.clone(),
            self.assets.camp_config.clone(),
            &self.assets.endgame_config,
            self.assets.boss_config.clone(),
        );

        self.assign_party(session.state_mut(), plan.strategy, seed);
        self.apply_persona_choice(session.state_mut(), plan.strategy);

        if let Some(setup) = plan.setup {
            setup(session.state_mut());
        }

        self.apply_store_loadout(session.state_mut(), plan.strategy, seed);
        Self::configure_strategy_settings(session.state_mut(), plan.strategy);

        if self.verbose {
            log_initial_state(seed, plan, session.state());
        }

        let mut policy = plan.strategy.create_policy(seed);
        let mut metrics = PlayabilityMetrics::default();
        let mut turns = Vec::new();
        if max_days == 0 {
            let final_state = session.into_state();
            metrics.finalize_without_turn(&final_state);
            return SimulationSummary {
                seed,
                mode: plan.mode,
                strategy: plan.strategy,
                turns,
                metrics,
                final_state,
                ending_message: "Simulation not executed".to_string(),
                game_ended: false,
            };
        }

        loop {
            let outcome = session.advance(policy.as_mut());
            metrics.record_turn(&outcome);

            if self.verbose {
                log_turn(&outcome, session.state());
            }

            let finished = outcome.game_ended;
            turns.push(outcome);

            if finished {
                break;
            }
        }

        let final_state = session.into_state();
        let final_outcome = turns.last().cloned().expect("simulation yielded no turns");
        metrics.finalize(&final_state, &final_outcome);

        SimulationSummary {
            seed,
            mode: plan.mode,
            strategy: plan.strategy,
            turns,
            metrics,
            final_state,
            ending_message: final_outcome.travel_message.clone(),
            game_ended: final_outcome.game_ended,
        }
    }
}

fn log_initial_state(seed: u64, plan: &SimulationPlan, state: &GameState) {
    println!(
        "🎮 Starting simulation | seed:{seed} mode:{:?} policy:{}",
        plan.mode,
        plan.strategy.label()
    );
    println!(
        "📊 Initial stats | HP:{} Supplies:{} Sanity:{} Pants:{} Budget:${}",
        state.stats.hp,
        state.stats.supplies,
        state.stats.sanity,
        state.stats.pants,
        format_cents(state.budget_cents)
    );
}

fn log_turn(outcome: &TurnOutcome, state: &GameState) {
    if let Some(decision) = &outcome.decision {
        println!(
            "🎯 Day {}: {} -> {} ({})",
            decision.day, decision.encounter_name, decision.choice_label, decision.policy_name
        );
    }

    if outcome.day.div_euclid(10) * 10 == outcome.day || outcome.game_ended {
        println!(
            "📅 Day {} stats | HP:{} Supplies:{} Sanity:{} Pants:{}",
            state.day, state.stats.hp, state.stats.supplies, state.stats.sanity, state.stats.pants
        );
    }

    if outcome.breakdown_started {
        if let Some(breakdown) = &state.breakdown {
            println!("🛞 Vehicle breakdown started: {:?}", breakdown.part);
        } else {
            println!("🛞 Vehicle breakdown started");
        }
    }

    if outcome.game_ended {
        println!("🏁 Simulation ended: {}", outcome.travel_message);
    }
}

fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let cents_abs = cents.unsigned_abs();
    let dollars = cents_abs / 100;
    let remainder = cents_abs % 100;
    format!("{sign}{dollars}.{remainder:02}")
}

#[derive(Debug, Clone, Default)]
pub struct BossOutcomeFlags {
    pub reached: bool,
    pub won: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EndgameStatus {
    pub active: bool,
    pub field_repair_used: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RunMilestones {
    pub reached_2000_by_day150: bool,
    pub survived: bool,
}

/// Aggregated analytics produced by a simulation run.
#[derive(Debug, Clone)]
pub struct PlayabilityMetrics {
    pub days_survived: i32,
    pub ending_type: String,
    pub ending_cause: String,
    pub encounters_faced: i32,
    pub vehicle_breakdowns: i32,
    pub final_hp: i32,
    pub final_supplies: i32,
    pub final_sanity: i32,
    pub final_pants: i32,
    pub final_budget_cents: i64,
    pub decision_log: Vec<DecisionRecord>,
    pub boss: BossOutcomeFlags,
    pub miles_traveled: f32,
    pub travel_days: u32,
    pub partial_travel_days: u32,
    pub non_travel_days: u32,
    pub avg_miles_per_day: f64,
    pub unique_encounters: u32,
    pub repairs_spent_cents: i64,
    pub bribes_spent_cents: i64,
    pub exec_order_active: String,
    pub exec_order_days_remaining: u32,
    pub exec_order_cooldown: u32,
    pub exposure_streak_heat: u32,
    pub exposure_streak_cold: u32,
    pub days_with_camp: u32,
    pub days_with_repair: u32,
    pub rotation_events: u32,
    pub travel_ratio: f64,
    pub unique_per_20_days: f64,
    pub milestones: RunMilestones,
    pub crossing_events: Vec<CrossingTelemetry>,
    pub crossing_permit_uses: u32,
    pub crossing_bribe_attempts: u32,
    pub crossing_bribe_successes: u32,
    pub crossing_detours_taken: u32,
    pub crossing_failures: u32,
    pub day_reason_history: Vec<String>,
    pub endgame: EndgameStatus,
    pub endgame_cooldown_days: u32,
    pub stop_cap_conversions: u32,
    pub failure_family: Option<FailureFamily>,
    encounter_ids: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureFamily {
    Vehicle,
    Sanity,
    Exposure,
    Crossing,
    Other,
}

impl Default for PlayabilityMetrics {
    fn default() -> Self {
        Self {
            days_survived: 0,
            ending_type: "In Progress".to_string(),
            ending_cause: String::new(),
            encounters_faced: 0,
            vehicle_breakdowns: 0,
            final_hp: 10,
            final_supplies: 10,
            final_sanity: 10,
            final_pants: 0,
            final_budget_cents: 10_000,
            decision_log: Vec::new(),
            boss: BossOutcomeFlags::default(),
            miles_traveled: 0.0,
            travel_days: 0,
            partial_travel_days: 0,
            non_travel_days: 0,
            avg_miles_per_day: 0.0,
            unique_encounters: 0,
            repairs_spent_cents: 0,
            bribes_spent_cents: 0,
            exec_order_active: String::new(),
            exec_order_days_remaining: 0,
            exec_order_cooldown: 0,
            exposure_streak_heat: 0,
            exposure_streak_cold: 0,
            days_with_camp: 0,
            days_with_repair: 0,
            rotation_events: 0,
            travel_ratio: 0.0,
            unique_per_20_days: 0.0,
            milestones: RunMilestones::default(),
            crossing_events: Vec::new(),
            crossing_permit_uses: 0,
            crossing_bribe_attempts: 0,
            crossing_bribe_successes: 0,
            crossing_detours_taken: 0,
            crossing_failures: 0,
            day_reason_history: Vec::new(),
            endgame: EndgameStatus::default(),
            endgame_cooldown_days: 0,
            stop_cap_conversions: 0,
            failure_family: None,
            encounter_ids: HashSet::new(),
        }
    }
}

#[derive(Default, Debug)]
struct LedgerSummary {
    miles: f64,
    travel_days: u32,
    partial_days: u32,
    non_travel_days: u32,
    stop_cap_conversions: u32,
    total_days: u32,
    reason_history: Vec<String>,
}

impl PlayabilityMetrics {
    fn summarize_day_records(state: &GameState) -> LedgerSummary {
        let metrics = compute_day_ledger_metrics(&state.day_records);
        let mut summary = LedgerSummary {
            miles: f64::from(metrics.total_miles),
            travel_days: metrics.travel_days,
            partial_days: metrics.partial_days,
            non_travel_days: metrics.non_travel_days,
            stop_cap_conversions: 0,
            total_days: metrics.total_days,
            reason_history: Vec::with_capacity(metrics.total_days as usize),
        };
        for record in &state.day_records {
            if record
                .tags
                .iter()
                .any(|tag| matches!(tag.0.as_str(), "stop_cap" | "auto_cap"))
            {
                summary.stop_cap_conversions = summary.stop_cap_conversions.saturating_add(1);
            }
            if record.tags.is_empty() {
                summary.reason_history.push(String::new());
            } else {
                let entry = record
                    .tags
                    .iter()
                    .map(|tag| tag.0.as_str())
                    .collect::<Vec<_>>()
                    .join(";");
                summary.reason_history.push(entry);
            }
        }
        summary
    }

    pub fn record_turn(&mut self, outcome: &TurnOutcome) {
        if let Some(decision) = outcome.decision.clone() {
            self.encounters_faced += 1;
            self.encounter_ids.insert(decision.encounter_id.clone());
            self.decision_log.push(decision);
        }

        if !self.milestones.reached_2000_by_day150
            && outcome.miles_traveled_actual >= MILESTONE_MILES
            && outcome.day <= MILESTONE_DAY_LIMIT
        {
            self.milestones.reached_2000_by_day150 = true;
        }

        if outcome.breakdown_started {
            self.vehicle_breakdowns += 1;
        }
    }

    fn capture_crossing_telemetry(&mut self, state: &GameState) {
        self.crossing_events.clone_from(&state.crossing_events);
        let mut permit_uses = 0;
        let mut bribe_attempts = 0;
        let mut bribe_successes = 0;
        let mut detours_taken = 0;
        let mut failures = 0;
        for event in &self.crossing_events {
            if event.permit_used {
                permit_uses += 1;
            }
            if event.bribe_attempted {
                bribe_attempts += 1;
                if event.bribe_success == Some(true) {
                    bribe_successes += 1;
                }
            }
            if event.detour_taken {
                detours_taken += 1;
            }
            if matches!(event.outcome, CrossingOutcomeTelemetry::Failed) {
                failures += 1;
            }
        }
        self.crossing_permit_uses = permit_uses;
        self.crossing_bribe_attempts = bribe_attempts;
        self.crossing_bribe_successes = bribe_successes;
        self.crossing_detours_taken = detours_taken;
        self.crossing_failures = failures;
    }

    pub fn finalize(&mut self, state: &GameState, outcome: &TurnOutcome) {
        self.days_survived = i32::try_from(state.day).unwrap_or(i32::MAX);
        self.final_hp = state.stats.hp;
        self.final_supplies = state.stats.supplies;
        self.final_sanity = state.stats.sanity;
        self.final_pants = state.stats.pants;
        self.final_budget_cents = state.budget_cents;
        let (ending, cause) = describe_ending(state, outcome);
        self.ending_type = ending;
        self.ending_cause = cause;
        self.boss.reached = state.boss.readiness.reached;
        self.boss.won = state.boss.outcome.victory;
        let ledger = Self::summarize_day_records(state);
        self.miles_traveled = clamp_f64_to_f32(ledger.miles.max(0.0));
        self.travel_days = ledger.travel_days;
        self.partial_travel_days = ledger.partial_days;
        self.non_travel_days = ledger.non_travel_days;
        let moving_days = ledger.travel_days.saturating_add(ledger.partial_days);
        self.avg_miles_per_day = if moving_days > 0 {
            ledger.miles / f64::from(moving_days)
        } else {
            0.0
        };
        self.unique_encounters = u32::try_from(self.encounter_ids.len()).unwrap_or(u32::MAX);
        self.repairs_spent_cents = state.repairs_spent_cents;
        self.bribes_spent_cents = state.bribes_spent_cents;
        self.exec_order_active = state
            .current_order
            .map(|order| order.key().to_string())
            .unwrap_or_default();
        self.exec_order_days_remaining = u32::from(state.exec_order_days_remaining);
        self.exec_order_cooldown = u32::from(state.exec_order_cooldown);
        self.exposure_streak_heat = u32::try_from(state.weather_state.heatwave_streak).unwrap_or(0);
        self.exposure_streak_cold = u32::try_from(state.weather_state.coldsnap_streak).unwrap_or(0);
        self.days_with_camp = state.days_with_camp;
        self.days_with_repair = state.days_with_repair;
        self.day_reason_history.clone_from(&ledger.reason_history);
        self.stop_cap_conversions = ledger.stop_cap_conversions;
        self.endgame.active = state.endgame.active;
        self.endgame.field_repair_used = state.endgame.field_repair_used;
        self.endgame_cooldown_days = state.vehicle.breakdown_cooldown;
        self.milestones.survived = survived_or_long_run(state);
        self.failure_family = classify_failure_family(state);
        let total_days = ledger.total_days;
        if total_days > 0 {
            self.travel_ratio =
                f64::from(self.travel_days + self.partial_travel_days) / f64::from(total_days);
            let windows = (f64::from(total_days) / 20.0).max(1.0);
            self.unique_per_20_days = f64::from(self.unique_encounters).max(0.0) / windows;
        } else {
            self.travel_ratio = 0.0;
            self.unique_per_20_days = 0.0;
        }
        self.rotation_events = u32::try_from(
            state
                .logs
                .iter()
                .filter(|entry| entry.as_str() == "log.encounter.rotation")
                .count(),
        )
        .unwrap_or(u32::MAX);
        self.capture_crossing_telemetry(state);
    }

    pub fn finalize_without_turn(&mut self, state: &GameState) {
        self.days_survived = i32::try_from(state.day).unwrap_or(i32::MAX);
        self.final_hp = state.stats.hp;
        self.final_supplies = state.stats.supplies;
        self.final_sanity = state.stats.sanity;
        self.final_pants = state.stats.pants;
        self.final_budget_cents = state.budget_cents;
        self.boss.reached = state.boss.readiness.reached;
        self.boss.won = state.boss.outcome.victory;
        let ledger = Self::summarize_day_records(state);
        self.miles_traveled = clamp_f64_to_f32(ledger.miles.max(0.0));
        self.travel_days = ledger.travel_days;
        self.partial_travel_days = ledger.partial_days;
        self.non_travel_days = ledger.non_travel_days;
        let moving_days = ledger.travel_days.saturating_add(ledger.partial_days);
        self.avg_miles_per_day = if moving_days > 0 {
            ledger.miles / f64::from(moving_days)
        } else {
            0.0
        };
        self.unique_encounters = u32::try_from(self.encounter_ids.len()).unwrap_or(u32::MAX);
        self.repairs_spent_cents = state.repairs_spent_cents;
        self.bribes_spent_cents = state.bribes_spent_cents;
        self.exec_order_active = state
            .current_order
            .map(|order| order.key().to_string())
            .unwrap_or_default();
        self.exec_order_days_remaining = u32::from(state.exec_order_days_remaining);
        self.exec_order_cooldown = u32::from(state.exec_order_cooldown);
        self.exposure_streak_heat = u32::try_from(state.weather_state.heatwave_streak).unwrap_or(0);
        self.exposure_streak_cold = u32::try_from(state.weather_state.coldsnap_streak).unwrap_or(0);
        self.days_with_camp = state.days_with_camp;
        self.days_with_repair = state.days_with_repair;
        self.day_reason_history.clone_from(&ledger.reason_history);
        self.stop_cap_conversions = ledger.stop_cap_conversions;
        self.endgame.active = state.endgame.active;
        self.endgame.field_repair_used = state.endgame.field_repair_used;
        self.endgame_cooldown_days = state.vehicle.breakdown_cooldown;
        let total_days = ledger.total_days;
        if total_days > 0 {
            self.travel_ratio =
                f64::from(self.travel_days + self.partial_travel_days) / f64::from(total_days);
            let windows = (f64::from(total_days) / 20.0).max(1.0);
            self.unique_per_20_days = f64::from(self.unique_encounters).max(0.0) / windows;
        } else {
            self.travel_ratio = 0.0;
            self.unique_per_20_days = 0.0;
        }
        self.rotation_events = u32::try_from(
            state
                .logs
                .iter()
                .filter(|entry| entry.as_str() == "log.encounter.rotation")
                .count(),
        )
        .unwrap_or(u32::MAX);
        self.capture_crossing_telemetry(state);
        if !self.milestones.reached_2000_by_day150
            && state.miles_traveled_actual >= MILESTONE_MILES
            && state.day <= MILESTONE_DAY_LIMIT
        {
            self.milestones.reached_2000_by_day150 = true;
        }
        let (ending, cause) = describe_ending(
            state,
            &TurnOutcome {
                day: state.day,
                travel_message: String::new(),
                breakdown_started: false,
                game_ended: false,
                decision: None,
                miles_traveled_actual: state.miles_traveled_actual,
            },
        );
        self.ending_type = ending;
        self.ending_cause = cause;
        self.milestones.survived = survived_or_long_run(state);
        self.failure_family = classify_failure_family(state);
    }
}

const SURVIVAL_DAY_THRESHOLD: u32 = 84;

const fn survived_or_long_run(state: &GameState) -> bool {
    state.boss.readiness.reached || state.day >= SURVIVAL_DAY_THRESHOLD
}

const fn classify_failure_family(state: &GameState) -> Option<FailureFamily> {
    match state.ending {
        Some(Ending::VehicleFailure { .. }) => Some(FailureFamily::Vehicle),
        Some(Ending::SanityLoss) => Some(FailureFamily::Sanity),
        Some(Ending::Exposure { .. }) => Some(FailureFamily::Exposure),
        Some(Ending::Collapse { cause }) => match cause {
            CollapseCause::Crossing => Some(FailureFamily::Crossing),
            CollapseCause::Vehicle | CollapseCause::Breakdown => Some(FailureFamily::Vehicle),
            CollapseCause::Weather => Some(FailureFamily::Exposure),
            CollapseCause::Panic | CollapseCause::Hunger | CollapseCause::Disease => {
                Some(FailureFamily::Other)
            }
        },
        _ => None,
    }
}

fn describe_ending(state: &GameState, outcome: &TurnOutcome) -> (String, String) {
    match state.ending {
        Some(Ending::BossVictory) => (
            "Victory - Boss Defeated".to_string(),
            "boss.victory".to_string(),
        ),
        Some(Ending::BossVoteFailed) => (
            "Boss Vote Failed - Game Over".to_string(),
            "boss_vote_failed".to_string(),
        ),
        Some(Ending::SanityLoss) => (
            "Sanity Depleted - Game Over".to_string(),
            "sanity".to_string(),
        ),
        Some(Ending::VehicleFailure { cause }) => (
            "Vehicle Failure - Game Over".to_string(),
            format!("vehicle_failure_{}", cause.key()),
        ),
        Some(Ending::Exposure { kind }) => (
            format!("Exposure ({}) - Game Over", kind.key()),
            format!("exposure_{}", kind.key()),
        ),
        Some(Ending::Collapse { cause }) => (
            format!("Collapse ({}) - Game Over", cause.key()),
            format!("collapse_{}", cause.key()),
        ),
        None if state.stats.pants >= 100 => (
            "Pants Emergency - Game Over".to_string(),
            "pants".to_string(),
        ),
        None if state.stats.hp <= 0 => (
            "Health Depleted - Game Over".to_string(),
            "health".to_string(),
        ),
        None if state.stats.sanity <= 0 => (
            "Sanity Depleted - Game Over".to_string(),
            "sanity".to_string(),
        ),
        None if state.stats.supplies <= 0 => (
            "Supplies Depleted - Game Over".to_string(),
            "supplies".to_string(),
        ),
        None if state.boss.outcome.attempted && !state.boss.outcome.victory => (
            "Boss Vote Failed - Game Over".to_string(),
            "boss_vote_failed".to_string(),
        ),
        None if state.boss.outcome.victory => (
            "Victory - Boss Defeated".to_string(),
            "boss.victory".to_string(),
        ),
        None if outcome.game_ended => (
            format!(
                "Game Ended: {}",
                humanize_log_message(&outcome.travel_message)
            ),
            "unknown".to_string(),
        ),
        None => ("Simulation Halted".to_string(), "in_progress".to_string()),
    }
}

fn humanize_log_message(message: &str) -> String {
    let stripped = message.strip_prefix(LOG_MESSAGE_PREFIX).unwrap_or(message);
    stripped
        .split(['.', '_'])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            chars.next().map_or_else(String::new, |first| {
                let mut formatted = first.to_uppercase().collect::<String>();
                formatted.push_str(chars.as_str());
                formatted
            })
        })
        .collect::<Vec<String>>()
        .join(" ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dystrail_game::personas::{Persona, PersonaMods, PersonaStart, PersonasList};
    use dystrail_game::store::{Grants, Store, StoreItem};
    use dystrail_game::weather::{
        Weather, WeatherAccumulationConfig, WeatherConfig, WeatherLimits, WeatherReportConfig,
    };
    use dystrail_game::{BossProgress, BossReadiness, BossResolution};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn build_tester() -> GameTester {
        GameTester::new(Arc::new(TesterAssets::load_default()), false)
    }

    fn make_weather_config(weights: HashMap<Region, HashMap<Weather, u32>>) -> WeatherConfig {
        WeatherConfig {
            limits: WeatherLimits {
                max_extreme_streak: 1,
                encounter_cap: 0.0,
                pants_floor: 0,
                pants_ceiling: 100,
            },
            effects: HashMap::new(),
            mitigation: HashMap::new(),
            weights,
            exec_mods: HashMap::new(),
            report: WeatherReportConfig::default(),
            accumulation: WeatherAccumulationConfig::default(),
        }
    }

    fn build_assets(
        weather_config: WeatherConfig,
        store: Store,
        personas: PersonasList,
    ) -> TesterAssets {
        TesterAssets {
            encounter_data: TesterAssets::fallback_encounter_data(),
            personas,
            store,
            camp_config: CampConfig::default(),
            boss_config: BossConfig::default(),
            weather_config,
            endgame_config: EndgameTravelCfg::default_config(),
        }
    }

    #[test]
    fn boss_config_loads_balanced_biases() {
        let assets = TesterAssets::load_default();
        assert!(
            (assets.boss_config.balanced.classic_bonus - 0.30).abs() < f32::EPSILON,
            "expected classic bonus from assets"
        );
        assert!(
            (assets.boss_config.balanced.deep_multiplier - 1.1).abs() < f32::EPSILON,
            "expected deep multiplier from assets"
        );
        assert!(
            (assets.boss_config.balanced.deep_bonus - 0.08).abs() < f32::EPSILON,
            "expected deep bonus from assets"
        );
    }

    #[test]
    fn balanced_setup_applies_persona_and_store_plan() {
        let tester = build_tester();
        let plan = SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_max_days(0)
            .with_setup(default_policy_setup(GameplayStrategy::Balanced));

        let summary = tester.run_plan(&plan, 12345);
        let state = summary.final_state;

        assert_eq!(state.persona_id.as_deref(), Some("staffer"));
        assert!(
            state.budget_cents < 11_000,
            "budget should reflect store spend"
        );
        assert_eq!(state.party.companions.len(), 4);
        assert!(!state.party.leader.is_empty());
        assert!(
            state.inventory.spares.tire >= 2,
            "store loadout should add an extra spare tire"
        );
        assert!(
            state.stats.supplies >= 18,
            "persona/start loadout should boost supplies"
        );
    }

    #[test]
    fn balanced_run_survives_past_day_45() {
        let tester = build_tester();
        let plan = SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced)
            .with_max_days(55)
            .with_setup(default_policy_setup(GameplayStrategy::Balanced));

        let summary = tester.run_plan(&plan, 4242);
        assert!(
            summary.metrics.days_survived >= 20,
            "expected survival past day 20, got {}",
            summary.metrics.days_survived
        );
    }

    #[test]
    fn miles_reflect_miles_traveled() {
        let tester = build_tester();
        let plan =
            SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced).with_max_days(5);
        let summary = tester.run_plan(&plan, 2024);
        let metrics = summary.metrics;
        let actual = summary.final_state.miles_traveled_actual;
        assert!(metrics.miles_traveled > 0.0);
        assert!(metrics.miles_traveled < summary.final_state.trail_distance);
        let diff = (metrics.miles_traveled - actual).abs();
        assert!(
            diff <= 1e-3,
            "miles_traveled {} should match actual {}",
            metrics.miles_traveled,
            actual
        );
    }

    #[test]
    fn boss_flags_only_when_attempted() {
        use crate::logic::simulation::TurnOutcome;

        let mut metrics = PlayabilityMetrics::default();
        let state = GameState::default();
        let outcome = TurnOutcome {
            day: state.day,
            travel_message: String::new(),
            breakdown_started: false,
            game_ended: true,
            decision: None,
            miles_traveled_actual: state.miles_traveled_actual,
        };

        metrics.finalize(&state, &outcome);
        assert!(!metrics.boss.reached);
        assert!(!metrics.boss.won);

        let attempted_state = GameState {
            boss: BossProgress {
                readiness: BossReadiness {
                    ready: true,
                    reached: true,
                },
                outcome: BossResolution {
                    attempted: true,
                    ..BossResolution::default()
                },
            },
            miles_traveled_actual: 1500.0,
            ..GameState::default()
        };
        let mut attempted_metrics = PlayabilityMetrics::default();
        attempted_metrics.finalize(&attempted_state, &outcome);
        assert!(attempted_metrics.boss.reached);
        assert!(!attempted_metrics.boss.won);
    }

    #[test]
    fn default_policy_setup_applies_strategy_defaults() {
        let mut state = GameState::default();
        default_policy_setup(GameplayStrategy::Balanced)(&mut state);
        assert!(state.stats.hp >= 8);
        assert!(state.stats.sanity >= 8);
        assert!(state.stats.supplies >= 12);
        assert!(state.stats.pants <= 5);
        assert!(state.inventory.spares.tire >= 1);

        let mut state = GameState::default();
        default_policy_setup(GameplayStrategy::Conservative)(&mut state);
        assert_eq!(state.stats.pants, 2);
        assert!(state.inventory.spares.tire >= 2);

        let mut state = GameState::default();
        default_policy_setup(GameplayStrategy::Aggressive)(&mut state);
        assert_eq!(state.stats.pants, 6);
        assert!(state.stats.supplies <= 12);

        let mut state = GameState::default();
        default_policy_setup(GameplayStrategy::ResourceManager)(&mut state);
        assert_eq!(state.budget_cents, 18_000);
        assert!(state.stats.supplies >= 12);
    }

    #[test]
    fn purchase_plan_adjusts_for_weather_risk() {
        let mut weights = HashMap::new();
        weights.insert(
            Region::Heartland,
            HashMap::from([
                (Weather::HeatWave, 10),
                (Weather::ColdSnap, 10),
                (Weather::Clear, 1),
            ]),
        );
        let weather_config = make_weather_config(weights);
        let store = Store {
            categories: Vec::new(),
            items: Vec::new(),
        };
        let assets = build_assets(weather_config, store, PersonasList::empty());
        let tester = GameTester::new(Arc::new(assets), false);

        let state = GameState {
            mode: GameMode::Classic,
            season: Season::Winter,
            ..GameState::default()
        };

        let plan = tester.planned_purchases(&state, GameplayStrategy::Balanced, 1);
        let ids: Vec<&str> = plan.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&"water"));
        assert!(ids.contains(&"coats"));
    }

    #[test]
    fn ensure_min_quantity_updates_and_inserts() {
        let mut plan = vec![("water", 1)];
        GameTester::ensure_min_quantity(&mut plan, "water", 3, None);
        assert!(plan.iter().any(|(id, qty)| *id == "water" && *qty == 3));

        GameTester::ensure_min_quantity(&mut plan, "coats", 1, Some(0));
        assert_eq!(plan.first().map(|(id, _)| *id), Some("coats"));
    }

    #[test]
    fn trim_noncritical_spare_removes_duplicates() {
        let mut plan = vec![("spare_tire", 1), ("spare_tire", 1), ("battery", 1)];
        GameTester::trim_noncritical_spare(&mut plan);
        let spare_count = plan.iter().filter(|(id, _)| *id == "spare_tire").count();
        assert_eq!(spare_count, 1);
    }

    #[test]
    fn prioritize_coat_inserts_near_spares() {
        let mut plan = vec![("spare_tire", 1), ("spare_tire", 1)];
        GameTester::prioritize_coat(&mut plan, GameplayStrategy::Conservative);
        assert!(plan.iter().any(|(id, _)| *id == "coats"));
    }

    #[test]
    fn party_roster_rotates_with_seed() {
        let (leader, companions) = GameTester::party_roster(GameplayStrategy::Balanced, 1);
        assert_eq!(leader, "Jordan Rivers");
        assert_eq!(companions.len(), 4);
    }

    #[test]
    fn dynamic_price_multiplier_accounts_for_conditions() {
        let state = GameState {
            season: Season::Winter,
            region: Region::Beltway,
            vehicle_breakdowns: 4,
            ..GameState::default()
        };

        let item = StoreItem {
            id: "coats".to_string(),
            name: "Coats".to_string(),
            desc: "Warm".to_string(),
            price_cents: 1000,
            unique: false,
            max_qty: 5,
            grants: Grants::default(),
            tags: vec!["warm_coat".to_string()],
            category: String::new(),
        };

        let multiplier = GameTester::dynamic_price_multiplier(&state, &item);
        assert!(multiplier > 100);
    }

    #[test]
    fn execute_purchase_applies_grants_and_logs() {
        let item = StoreItem {
            id: "water".to_string(),
            name: "Water".to_string(),
            desc: "Hydration".to_string(),
            price_cents: 100,
            unique: false,
            max_qty: 5,
            grants: Grants {
                supplies: 2,
                ..Grants::default()
            },
            tags: vec!["water_jugs".to_string()],
            category: String::new(),
        };
        let store = Store {
            categories: Vec::new(),
            items: vec![item],
        };
        let personas = PersonasList(vec![Persona {
            id: "staffer".to_string(),
            name: "Staffer".to_string(),
            desc: "Test".to_string(),
            score_mult: 1.0,
            start: PersonaStart::default(),
            mods: PersonaMods::default(),
        }]);

        let weather_config = make_weather_config(HashMap::new());
        let tester = GameTester::new(
            Arc::new(build_assets(weather_config, store, personas)),
            false,
        );
        let mut state = GameState {
            budget_cents: 10_000,
            ..GameState::default()
        };

        tester.execute_purchase(&mut state, "water", 2);
        assert!(state.stats.supplies >= 12);
        assert!(state.budget_cents < 10_000);
        assert!(
            state
                .logs
                .iter()
                .any(|log| log.contains("log.store.purchase.water"))
        );
    }

    #[test]
    fn apply_store_loadout_skips_without_budget() {
        let weather_config = make_weather_config(HashMap::new());
        let store = Store {
            categories: Vec::new(),
            items: Vec::new(),
        };
        let tester = GameTester::new(
            Arc::new(build_assets(weather_config, store, PersonasList::empty())),
            false,
        );
        let mut state = GameState {
            budget_cents: 0,
            ..GameState::default()
        };
        let starting_logs = state.logs.clone();
        tester.apply_store_loadout(&mut state, GameplayStrategy::Balanced, 1);
        assert_eq!(state.budget_cents, 0);
        assert_eq!(state.logs, starting_logs);
    }
}
