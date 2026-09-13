use super::*;

fn health_seed_between(low: f32, high: f32) -> u64 {
    (0..100_000)
        .find(|seed| {
            let bundle = RngBundle::from_user_seed(*seed);
            let roll = bundle.health().r#gen::<f32>();
            roll >= low && roll < high
        })
        .expect("the fixed seed range contains the required illness roll")
}

fn illness_state(seed: u64, protected: bool) -> GameState {
    let mut state = GameState::default();
    state.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(seed)));
    if protected {
        state.inventory.tags.insert("plague_resist".into());
    }
    state
}

#[test]
fn masks_reduce_new_illness_risk_without_erasing_unprotected_risk() {
    let seed = health_seed_between(DISEASE_DAILY_CHANCE / 2.0, DISEASE_DAILY_CHANCE);
    let mut unprotected = illness_state(seed, false);
    let mut protected = illness_state(seed, true);
    let before = protected.clone();
    unprotected.roll_daily_illness();
    protected.roll_daily_illness();

    assert!(unprotected.illness_days_remaining > 0);
    assert!(unprotected.day_state.rest.rest_requested);
    assert_eq!(protected.illness_days_remaining, 0);
    assert!(!protected.day_state.rest.rest_requested);
    assert_eq!(protected.stats, before.stats);
    assert_eq!(protected.day, before.day);
    assert_eq!(
        protected.continuity.clock_minutes,
        before.continuity.clock_minutes
    );
    assert_eq!(
        protected.miles_traveled_actual.to_bits(),
        before.miles_traveled_actual.to_bits()
    );
    assert!(protected.inventory.has_tag("plague_resist"));
}

#[test]
fn masks_do_not_guarantee_immunity_or_treat_existing_illness() {
    let seed = health_seed_between(0.0, DISEASE_DAILY_CHANCE / 2.0);
    let mut protected = illness_state(seed, true);
    let before = protected.stats.clone();
    protected.roll_daily_illness();
    assert!(protected.illness_days_remaining > 0);
    assert_eq!(protected.stats.hp, before.hp - DISEASE_HP_PENALTY);
    assert_eq!(
        protected.stats.sanity,
        before.sanity - DISEASE_SANITY_PENALTY
    );
    let mut unprotected = illness_state(seed, false);
    unprotected.illness_days_remaining = 2;
    protected.illness_days_remaining = 2;
    protected.stats = unprotected.stats.clone();
    let before = protected.stats.clone();
    unprotected.roll_daily_illness();
    protected.roll_daily_illness();
    assert_eq!(protected.illness_days_remaining, 1);
    assert_eq!(protected.stats, unprotected.stats);
    assert_eq!(protected.stats.hp, before.hp - DISEASE_TICK_HP_LOSS);
    assert!(protected.day_state.rest.rest_requested);
}

#[test]
fn masks_protection_survives_reload_and_cooldown_still_prevents_rolls() {
    let seed = health_seed_between(DISEASE_DAILY_CHANCE / 2.0, DISEASE_DAILY_CHANCE);
    let original = illness_state(seed, true);
    let mut restored: GameState =
        serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
    restored.attach_rng_bundle(Rc::new(RngBundle::from_user_seed(seed)));
    restored.roll_daily_illness();
    assert_eq!(restored.illness_days_remaining, 0);
    assert_eq!(restored.stats, original.stats);
    restored.disease_cooldown = 2;
    let before = restored.stats.clone();
    let draws = restored.rng_bundle.as_ref().unwrap().health().draws();
    restored.roll_daily_illness();
    assert_eq!(restored.disease_cooldown, 1);
    assert_eq!(
        restored.rng_bundle.as_ref().unwrap().health().draws(),
        draws
    );
    assert_eq!(restored.stats, before);
}
