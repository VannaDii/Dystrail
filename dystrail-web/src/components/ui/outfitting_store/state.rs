use crate::game::{
    GameState,
    store::{Cart, Grants, Store},
};
use thiserror::Error;
use yew::prelude::*;

/// The different screens within the store
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreScreen {
    /// Main store menu showing categories
    Home,
    /// Category view showing items in a category
    Category(String),
    /// Quantity selection for a specific item
    #[cfg(any(target_arch = "wasm32", test))]
    QuantityPrompt(String),
    /// Cart/checkout view
    Cart,
}

/// Store interface state
#[derive(Clone)]
pub struct StoreState {
    pub store_data: Store,
    pub cart: Cart,
    pub current_screen: StoreScreen,
    pub focus_idx: u8,
    pub discount_pct: f64,
}

impl Default for StoreState {
    fn default() -> Self {
        Self {
            store_data: Store {
                categories: vec![],
                items: vec![],
            },
            cart: Cart::new(),
            current_screen: StoreScreen::Home,
            focus_idx: 1,
            discount_pct: 0.0,
        }
    }
}

#[derive(Properties, Clone)]
pub struct OutfittingStoreProps {
    /// Current game state for budget and persona info
    pub game_state: GameState,
    /// Callback when the player proceeds past the store
    pub on_continue: Callback<(GameState, Grants, Vec<String>)>,
}

impl PartialEq for OutfittingStoreProps {
    fn eq(&self, other: &Self) -> bool {
        self.game_state.budget_cents == other.game_state.budget_cents
            && self.game_state.persona_id == other.game_state.persona_id
            && self.game_state.mods.store_discount_pct == other.game_state.mods.store_discount_pct
    }
}

pub(super) fn set_screen(state: &UseStateHandle<StoreState>, screen: StoreScreen) {
    let new_state = screen_state(state, screen);
    state.set(new_state);
}

fn screen_state(state: &StoreState, screen: StoreScreen) -> StoreState {
    let mut new_state = state.clone();
    new_state.current_screen = screen;
    new_state.focus_idx = 1;
    new_state
}

#[derive(Debug, Error)]
pub(super) enum StoreLoadError {
    #[error("JSON parsing error: {0}")]
    Parse(#[from] serde_json::Error),
}

/// Load store data from embedded JSON.
pub(super) fn load_store_data() -> Result<Store, StoreLoadError> {
    let text = include_str!("../../../../static/assets/data/store.json");
    let store: Store = serde_json::from_str(text)?;
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_store_data_returns_categories() {
        let store = load_store_data().expect("store data");
        assert!(!store.categories.is_empty());
    }

    #[test]
    fn default_store_state_starts_on_home() {
        let state = StoreState::default();
        assert!(matches!(state.current_screen, StoreScreen::Home));
        assert_eq!(state.focus_idx, 1);
    }

    #[test]
    fn set_screen_resets_focus_index() {
        let state = StoreState::default();
        let next_state = screen_state(&state, StoreScreen::Cart);
        assert!(matches!(next_state.current_screen, StoreScreen::Cart));
        assert_eq!(next_state.focus_idx, 1);
    }

    #[test]
    fn store_props_equality_compares_budget_and_persona() {
        let base_state = GameState {
            budget_cents: 100,
            persona_id: Some(String::from("organizer")),
            ..GameState::default()
        };
        let props_a = OutfittingStoreProps {
            game_state: base_state.clone(),
            on_continue: Callback::noop(),
        };
        let props_b = OutfittingStoreProps {
            game_state: base_state,
            on_continue: Callback::noop(),
        };
        assert!(props_a == props_b);
    }
}
