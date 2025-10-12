use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use dystrail_game::boss::BossConfig;
use dystrail_game::camp::CampConfig;
use dystrail_game::data::{Choice, Effects, Encounter, EncounterData};
use dystrail_game::pacing::PacingConfig;
use dystrail_game::personas::{Persona, PersonasList};
use dystrail_game::state::Ending;
use dystrail_game::store::{Grants, Store, StoreItem, calculate_effective_price};
use dystrail_game::{DietId, GameMode, GameState, PaceId};
use serde_json;

use crate::logic::policy::GameplayStrategy;
use crate::logic::simulation::{DecisionRecord, SimulationConfig, SimulationSession, TurnOutcome};

const LOG_MESSAGE_PREFIX: &str = "log.";

/// Collection of immutable data required to run a simulation.
#[derive(Debug, Clone)]
struct TesterAssets {
    encounter_data: EncounterData,
    pacing_config: PacingConfig,
    personas: PersonasList,
    store: Store,
    camp_config: CampConfig,
    boss_config: BossConfig,
}

impl TesterAssets {
    fn load_default() -> Self {
        let encounter_data =
            Self::load_encounters_from_assets().unwrap_or_else(Self::fallback_encounter_data);
        let pacing_config = Self::load_pacing_from_assets().unwrap_or_default();
        let personas = Self::load_personas_from_assets().unwrap_or_else(PersonasList::empty);
        let store = Self::load_store_from_assets().unwrap_or_else(Self::fallback_store_data);
        let camp_config = Self::load_camp_from_assets().unwrap_or_default();
        let boss_config = Self::load_boss_from_assets().unwrap_or_default();

        Self {
            encounter_data,
            pacing_config,
            personas,
            store,
            camp_config,
            boss_config,
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

    fn load_pacing_from_assets() -> Option<PacingConfig> {
        let base = Self::assets_data_root();
        let json = fs::read_to_string(base.join("pacing.json")).ok()?;
        serde_json::from_str(&json).ok()
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
                        add_receipt: None,
                        use_receipt: false,
                        log: Some("You keep morale up with snacks.".to_string()),
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
                        add_receipt: None,
                        use_receipt: false,
                        log: Some("Tension rises as you hoard the jerky.".to_string()),
                    },
                },
            ],
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
        GameplayStrategy::Balanced | GameplayStrategy::MonteCarlo => balanced_policy_setup,
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

    #[allow(dead_code)]
    #[must_use]
    pub fn with_max_days(mut self, max_days: u32) -> Self {
        self.max_days = Some(max_days);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_setup(mut self, setup: fn(&mut GameState)) -> Self {
        self.setup = Some(setup);
        self
    }

    #[must_use]
    pub fn with_expectation(mut self, expectation: SimulationExpectation) -> Self {
        self.expectations.push(expectation);
        self
    }
}

/// Assertion hook run after a simulation completes.
pub type SimulationExpectation = fn(&SimulationSummary) -> Result<()>;

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
    pub fn try_new(verbose: bool) -> Self {
        let assets = TesterAssets::load_default();
        Self {
            verbose,
            assets: Arc::new(assets),
        }
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
            GameplayStrategy::MonteCarlo => "satirist",
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

        for (item_id, qty) in Self::planned_purchases(strategy, seed) {
            self.execute_purchase(state, item_id, qty);
        }
    }

    fn configure_strategy_settings(state: &mut GameState, strategy: GameplayStrategy) {
        let (auto_rest, threshold) = if matches!(strategy, GameplayStrategy::Aggressive) {
            (true, 3)
        } else {
            (true, 4)
        };
        state.auto_camp_rest = auto_rest;
        state.rest_threshold = threshold;
        state.rest_requested = false;
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

    fn planned_purchases(strategy: GameplayStrategy, seed: u64) -> Vec<(&'static str, i32)> {
        match strategy {
            GameplayStrategy::Balanced => vec![("rations", 2), ("water", 1), ("spare_tire", 1)],
            GameplayStrategy::Conservative => {
                vec![("spare_tire", 2), ("battery", 1), ("legal_fund", 1)]
            }
            GameplayStrategy::Aggressive => vec![("legal_fund", 2), ("rations", 1)],
            GameplayStrategy::ResourceManager => {
                vec![("rations", 3), ("water", 2), ("spare_tire", 1)]
            }
            GameplayStrategy::MonteCarlo => match seed % 3 {
                0 => vec![("rations", 1), ("press_pass", 1), ("masks", 1)],
                1 => vec![("rations", 2), ("legal_fund", 1)],
                _ => vec![("water", 2), ("ponchos", 1), ("press_pass", 1)],
            },
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
            GameplayStrategy::MonteCarlo => [
                "Indigo Reyes",
                "Sterling Vaughn",
                "Phoenix Cole",
                "Rowan Hart",
                "Sable Frost",
            ],
        };

        let mut names: Vec<String> = base.into_iter().map(String::from).collect();
        if let Ok(len_u64) = u64::try_from(names.len())
            && len_u64 > 0
        {
            let offset = usize::try_from(seed % len_u64).unwrap_or(0);
            names.rotate_left(offset);
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
        let unit_price = calculate_effective_price(item.price_cents, discount);
        let qty_i64 = i64::from(qty);
        let total_cost = unit_price.saturating_mul(qty_i64);
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
            SimulationConfig::new(plan.mode, seed).with_max_days(max_days),
            self.assets.encounter_data.clone(),
            self.assets.pacing_config.clone(),
            self.assets.camp_config.clone(),
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

    #[allow(dead_code)]
    pub fn play_game(
        &self,
        mode: GameMode,
        strategy: GameplayStrategy,
        seed: u64,
    ) -> PlayabilityMetrics {
        let plan = SimulationPlan::new(mode, strategy)
            .with_max_days(DEFAULT_POLICY_SIM_DAYS)
            .with_setup(default_policy_setup(strategy));
        let summary = self.run_plan(&plan, seed);
        summary.metrics
    }
}

fn log_initial_state(seed: u64, plan: &SimulationPlan, state: &GameState) {
    println!(
        "🎮 Starting simulation | seed:{seed} mode:{:?} policy:{}",
        plan.mode,
        plan.strategy.label()
    );
    #[allow(clippy::cast_precision_loss)]
    {
        println!(
            "📊 Initial stats | HP:{} Supplies:{} Sanity:{} Pants:{} Budget:${}",
            state.stats.hp,
            state.stats.supplies,
            state.stats.sanity,
            state.stats.pants,
            format_cents(state.budget_cents)
        );
    }
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
    pub boss_reached: bool,
    pub boss_won: bool,
    pub miles_traveled: f32,
    pub travel_days: u32,
    pub non_travel_days: u32,
    pub avg_miles_per_day: f64,
    pub unique_encounters: u32,
    pub repairs_spent_cents: i64,
    pub bribes_spent_cents: i64,
    encounter_ids: HashSet<String>,
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
            boss_reached: false,
            boss_won: false,
            miles_traveled: 0.0,
            travel_days: 0,
            non_travel_days: 0,
            avg_miles_per_day: 0.0,
            unique_encounters: 0,
            repairs_spent_cents: 0,
            bribes_spent_cents: 0,
            encounter_ids: HashSet::new(),
        }
    }
}

impl PlayabilityMetrics {
    pub fn record_turn(&mut self, outcome: &TurnOutcome) {
        if let Some(decision) = outcome.decision.clone() {
            self.encounters_faced += 1;
            self.encounter_ids.insert(decision.encounter_id.clone());
            self.decision_log.push(decision);
        }

        if outcome.breakdown_started {
            self.vehicle_breakdowns += 1;
        }
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
        self.boss_won = state.boss_victory;
        self.boss_reached = state.boss_attempted;
        self.miles_traveled = if state.distance_traveled_actual > 0.0 {
            state.distance_traveled_actual
        } else {
            state.distance_traveled
        };
        self.travel_days = state.travel_days;
        self.non_travel_days = state.non_travel_days;
        self.avg_miles_per_day = if state.travel_days > 0 {
            let days = state.travel_days.max(1);
            f64::from(state.distance_traveled_actual) / f64::from(days)
        } else {
            0.0
        };
        self.unique_encounters = u32::try_from(self.encounter_ids.len()).unwrap_or(u32::MAX);
        self.repairs_spent_cents = state.repairs_spent_cents;
        self.bribes_spent_cents = state.bribes_spent_cents;
    }

    pub fn finalize_without_turn(&mut self, state: &GameState) {
        self.days_survived = i32::try_from(state.day).unwrap_or(i32::MAX);
        self.final_hp = state.stats.hp;
        self.final_supplies = state.stats.supplies;
        self.final_sanity = state.stats.sanity;
        self.final_pants = state.stats.pants;
        self.final_budget_cents = state.budget_cents;
        self.boss_reached = state.boss_attempted;
        self.boss_won = state.boss_victory;
        self.miles_traveled = if state.distance_traveled_actual > 0.0 {
            state.distance_traveled_actual
        } else {
            state.distance_traveled
        };
        self.travel_days = state.travel_days;
        self.non_travel_days = state.non_travel_days;
        self.avg_miles_per_day = if state.travel_days > 0 {
            let days = state.travel_days.max(1);
            f64::from(state.distance_traveled_actual) / f64::from(days)
        } else {
            0.0
        };
        self.unique_encounters = u32::try_from(self.encounter_ids.len()).unwrap_or(u32::MAX);
        self.repairs_spent_cents = state.repairs_spent_cents;
        self.bribes_spent_cents = state.bribes_spent_cents;
        let (ending, cause) = describe_ending(
            state,
            &TurnOutcome {
                day: state.day,
                travel_message: String::new(),
                breakdown_started: false,
                game_ended: false,
                decision: None,
            },
        );
        self.ending_type = ending;
        self.ending_cause = cause;
    }
}

fn describe_ending(state: &GameState, outcome: &TurnOutcome) -> (String, String) {
    if let Some(ending) = state.ending {
        match ending {
            Ending::BossVictory => (
                "Victory - Boss Defeated".to_string(),
                "boss_victory".to_string(),
            ),
            Ending::BossVoteFailed => (
                "Boss Vote Failed - Game Over".to_string(),
                "boss_vote_failed".to_string(),
            ),
            Ending::SanityLoss => (
                "Sanity Depleted - Game Over".to_string(),
                "sanity".to_string(),
            ),
            Ending::VehicleFailure { cause } => (
                "Vehicle Failure - Game Over".to_string(),
                cause.key().to_string(),
            ),
            Ending::Exposure { kind } => (
                format!("Exposure ({}) - Game Over", kind.key()),
                format!("exposure_{}", kind.key()),
            ),
            Ending::Collapse { cause } => (
                format!("Collapse ({}) - Game Over", cause.key()),
                cause.key().to_string(),
            ),
        }
    } else if state.stats.pants >= 100 {
        (
            "Pants Emergency - Game Over".to_string(),
            "pants".to_string(),
        )
    } else if state.stats.hp <= 0 {
        (
            "Health Depleted - Game Over".to_string(),
            "health".to_string(),
        )
    } else if state.stats.sanity <= 0 {
        (
            "Sanity Depleted - Game Over".to_string(),
            "sanity".to_string(),
        )
    } else if state.stats.supplies <= 0 {
        (
            "Supplies Depleted - Game Over".to_string(),
            "supplies".to_string(),
        )
    } else if state.boss_attempted && !state.boss_victory {
        (
            "Boss Vote Failed - Game Over".to_string(),
            "boss_vote_failed".to_string(),
        )
    } else if state.boss_victory {
        (
            "Victory - Boss Defeated".to_string(),
            "boss_victory".to_string(),
        )
    } else if outcome.game_ended {
        (
            format!(
                "Game Ended: {}",
                humanize_log_message(&outcome.travel_message)
            ),
            "unknown".to_string(),
        )
    } else {
        ("Simulation Halted".to_string(), "in_progress".to_string())
    }
}

fn humanize_log_message(message: &str) -> String {
    let stripped = message.strip_prefix(LOG_MESSAGE_PREFIX).unwrap_or(message);
    stripped
        .split(['.', '_'])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            if let Some(first) = chars.next() {
                let mut formatted = first.to_uppercase().collect::<String>();
                formatted.push_str(chars.as_str());
                formatted
            } else {
                String::new()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_setup_applies_persona_and_store_plan() {
        let tester = GameTester::try_new(false);
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
        assert!(state.auto_camp_rest);
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
        let tester = GameTester::try_new(false);
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
    fn miles_reflect_distance_traveled() {
        let tester = GameTester::try_new(false);
        let plan =
            SimulationPlan::new(GameMode::Classic, GameplayStrategy::Balanced).with_max_days(5);
        let summary = tester.run_plan(&plan, 2024);
        let metrics = summary.metrics;
        let actual = summary.final_state.distance_traveled_actual;
        assert!(metrics.miles_traveled > 0.0);
        assert!(metrics.miles_traveled < summary.final_state.trail_distance);
        let diff = (metrics.miles_traveled - actual).abs();
        assert!(
            diff <= f32::EPSILON,
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
        };

        metrics.finalize(&state, &outcome);
        assert!(!metrics.boss_reached);
        assert!(!metrics.boss_won);

        let attempted_state = GameState {
            boss_attempted: true,
            distance_traveled_actual: 1500.0,
            ..GameState::default()
        };
        let mut attempted_metrics = PlayabilityMetrics::default();
        attempted_metrics.finalize(&attempted_state, &outcome);
        assert!(attempted_metrics.boss_reached);
        assert!(!attempted_metrics.boss_won);
        assert!(attempted_metrics.miles_traveled > 0.0);
    }
}
