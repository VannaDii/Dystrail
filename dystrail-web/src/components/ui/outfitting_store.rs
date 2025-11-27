//! Outfitting Store component - Oregon Trail style store with numbered menu navigation.
//!
//! This component handles the store interface where players can purchase supplies,
//! vehicle spares, PPE, and documents before starting their journey.

use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{KeyboardEvent, window};
use yew::prelude::*;

use crate::a11y::set_status;
use crate::dom;
use crate::game::{
    GameState,
    store::{Cart, Grants, Store, StoreItem, calculate_cart_total, calculate_effective_price},
};
use crate::i18n;
use crate::input::{numeric_code_to_index, numeric_key_to_index};
use thiserror::Error;

/// The different screens within the store
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreScreen {
    /// Main store menu showing categories
    Home,
    /// Category view showing items in a category
    Category(String), // category_id
    /// Quantity selection for a specific item
    QuantityPrompt(String), // item_id
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
        // Compare relevant fields for equality
        self.game_state.budget_cents == other.game_state.budget_cents
            && self.game_state.persona_id == other.game_state.persona_id
            && self.game_state.mods.store_discount_pct == other.game_state.mods.store_discount_pct
        // Note: Callbacks are not compared as they don't implement PartialEq
    }
}

fn set_screen(state: &UseStateHandle<StoreState>, screen: StoreScreen) {
    let mut new_state = (**state).clone();
    new_state.current_screen = screen;
    new_state.focus_idx = 1;
    state.set(new_state);
}

#[function_component(OutfittingStore)]
pub fn outfitting_store(props: &OutfittingStoreProps) -> Html {
    let store_state = use_state(StoreState::default);
    let list_ref = use_node_ref();
    let _live_region_ref = use_node_ref();

    // Calculate discount from persona
    let discount_pct = f64::from(props.game_state.mods.store_discount_pct);

    // Load store data on mount
    {
        let store_state = store_state.clone();
        use_effect_with((), move |()| {
            wasm_bindgen_futures::spawn_local(async move {
                match load_store_data().await {
                    Ok(store_data) => {
                        let mut state = (*store_state).clone();
                        state.store_data = store_data;
                        state.discount_pct = discount_pct;
                        state.cart.total_cents =
                            calculate_cart_total(&state.cart, &state.store_data, discount_pct);
                        store_state.set(state);
                    }
                    Err(e) => {
                        dom::console_error(&format!("Failed to load store data: {e}"));
                    }
                }
            });
        });
    }

    // Update cart total when cart changes
    {
        let store_state = store_state.clone();
        use_effect_with(store_state.cart.clone(), move |cart| {
            let mut state = (*store_state).clone();
            state.cart.total_cents =
                calculate_cart_total(cart, &state.store_data, state.discount_pct);
            store_state.set(state);
        });
    }

    // Handle keyboard navigation and activation
    let on_keydown = {
        let store_state = store_state.clone();
        let props = props.clone();
        Callback::from(move |e: KeyboardEvent| {
            let key = e.key();
            let state = (*store_state).clone();

            // Handle numeric key activation
            if let Some(n) = numeric_key_to_index(&key).or_else(|| numeric_code_to_index(&e.code()))
            {
                handle_menu_selection(n, &state, &store_state, &props);
                e.prevent_default();
                return;
            }

            // Handle arrow key navigation
            match key.as_str() {
                "Enter" | " " => {
                    handle_menu_selection(state.focus_idx, &state, &store_state, &props);
                    e.prevent_default();
                }
                "Escape" => {
                    handle_back_navigation(&state, &store_state);
                    e.prevent_default();
                }
                "ArrowDown" => {
                    let max_idx = get_max_menu_index(&state);
                    let next = if state.focus_idx >= max_idx {
                        1
                    } else {
                        state.focus_idx + 1
                    };
                    let mut new_state = state;
                    new_state.focus_idx = next;
                    store_state.set(new_state);
                    e.prevent_default();
                }
                "ArrowUp" => {
                    let max_idx = get_max_menu_index(&state);
                    let prev = if state.focus_idx <= 1 {
                        max_idx
                    } else {
                        state.focus_idx - 1
                    };
                    let mut new_state = state;
                    new_state.focus_idx = prev;
                    store_state.set(new_state);
                    e.prevent_default();
                }
                _ => {}
            }
        })
    };

    // Focus management effect
    {
        let list_ref = list_ref.clone();
        let focus_idx = store_state.focus_idx;
        use_effect_with(focus_idx, move |idx| {
            if let Some(list) = list_ref.cast::<web_sys::Element>() {
                let sel = format!("[role='menuitem'][data-key='{idx}']");
                if let Ok(Some(el)) = list.query_selector(&sel) {
                    let _ = el
                        .dyn_into::<web_sys::HtmlElement>()
                        .ok()
                        .map(|e| e.focus());
                }
            }
        });
    }

    // Render based on current screen
    match &store_state.current_screen {
        StoreScreen::Home => {
            render_home_screen(&store_state, &props.game_state, &list_ref, &on_keydown)
        }
        StoreScreen::Category(category_id) => render_category_screen(
            category_id,
            &store_state,
            &props.game_state,
            &list_ref,
            &on_keydown,
        ),
        StoreScreen::QuantityPrompt(item_id) => render_quantity_screen(
            item_id,
            &store_state,
            &props.game_state,
            &list_ref,
            &on_keydown,
        ),
        StoreScreen::Cart => {
            render_cart_screen(&store_state, &props.game_state, &list_ref, &on_keydown)
        }
    }
}

#[derive(Debug, Error)]
enum StoreLoadError {
    #[error("Request failed: {0}")]
    Request(String),
    #[error("Response was not valid UTF-8")]
    Utf8,
    #[error("JSON parsing error: {0}")]
    Parse(#[from] serde_json::Error),
}

/// Load store data from JSON file
#[allow(clippy::future_not_send)]
async fn load_store_data() -> Result<Store, StoreLoadError> {
    let response = dom::fetch_response("/static/assets/data/store.json")
        .await
        .map_err(|err| StoreLoadError::Request(dom::js_error_message(&err)))?;

    if !response.ok() {
        return Err(StoreLoadError::Request(format!(
            "HTTP {status}: {status_text}",
            status = response.status(),
            status_text = response.status_text()
        )));
    }

    let text_js = JsFuture::from(
        response
            .text()
            .map_err(|err| StoreLoadError::Request(dom::js_error_message(&err)))?,
    )
    .await
    .map_err(|err| StoreLoadError::Request(dom::js_error_message(&err)))?;

    let text = text_js.as_string().ok_or(StoreLoadError::Utf8)?;

    let store: Store = serde_json::from_str(&text)?;
    Ok(store)
}

/// Handle menu item selection
fn handle_menu_selection(
    index: u8,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    match &state.current_screen {
        StoreScreen::Home => {
            match index {
                1..=4 => {
                    // Navigate to category
                    if let Some(category) = state.store_data.categories.get((index - 1) as usize) {
                        let mut new_state = state.clone();
                        new_state.current_screen = StoreScreen::Category(category.id.clone());
                        new_state.focus_idx = 1;
                        store_state.set(new_state);

                        // Announce navigation
                        let category_name = i18n::t(&format!(
                            "store.categories.{category_id}",
                            category_id = category.id
                        ));
                        announce_to_screen_reader(&category_name);
                    }
                }
                5 => {
                    // View cart
                    let mut new_state = state.clone();
                    new_state.current_screen = StoreScreen::Cart;
                    new_state.focus_idx = 1;
                    store_state.set(new_state);

                    announce_to_screen_reader(&i18n::t("store.menu.view_cart"));
                }
                0 => {
                    // Continue (if cart total allows)
                    let remaining_budget = props.game_state.budget_cents - state.cart.total_cents;
                    if remaining_budget >= 0 {
                        handle_checkout(state, props);
                    } else {
                        announce_to_screen_reader(&i18n::t("store.alerts.over_budget"));
                    }
                }
                _ => {}
            }
        }
        StoreScreen::Category(category_id) => {
            if index == 0 {
                // Back to home
                handle_back_navigation(state, store_state);
            } else if let Some(item) = state
                .store_data
                .categories
                .iter()
                .find(|c| c.id == *category_id)
                .and_then(|category| category.items.get((index - 1) as usize))
            {
                let mut new_state = state.clone();
                new_state.current_screen = StoreScreen::QuantityPrompt(item.id.clone());
                new_state.focus_idx = 1;
                store_state.set(new_state);

                let item_name = i18n::t(&format!("store.items.{}.name", item.id));
                announce_to_screen_reader(&item_name);
            }
        }
        StoreScreen::QuantityPrompt(item_id) => {
            handle_quantity_selection(index, item_id, state, store_state, props);
        }
        StoreScreen::Cart => {
            handle_cart_selection(index, state, store_state, props);
        }
    }
}

/// Handle navigation back to previous screen
fn handle_back_navigation(state: &StoreState, store_state: &UseStateHandle<StoreState>) {
    let mut new_state = state.clone();
    match &state.current_screen {
        StoreScreen::Category(_) | StoreScreen::Cart => {
            new_state.current_screen = StoreScreen::Home;
        }
        StoreScreen::QuantityPrompt(_) => {
            // Go back to the category (need to find which category contains this item)
            // For now, go back to home
            new_state.current_screen = StoreScreen::Home;
        }
        StoreScreen::Home => {
            // Already at home, no action
            return;
        }
    }
    new_state.focus_idx = 1;
    store_state.set(new_state);

    announce_to_screen_reader(&i18n::t("store.menu.back"));
}

/// Get the maximum menu index for the current screen
fn get_max_menu_index(state: &StoreState) -> u8 {
    match &state.current_screen {
        StoreScreen::Home => 5, // Categories 1-4, cart 5, continue 0
        StoreScreen::Category(category_id) => state
            .store_data
            .categories
            .iter()
            .find(|c| c.id == *category_id)
            .map_or(1, |category| {
                u8::try_from(category.items.len()).unwrap_or(255)
            }),
        StoreScreen::QuantityPrompt(_) => 4, // Add +1, +5, Remove -1, Remove All (1-4), back 0
        StoreScreen::Cart => {
            if state.cart.lines.is_empty() {
                1 // Only checkout/back
            } else {
                u8::try_from(state.cart.lines.len()).unwrap_or(255) // Cart items 1-N, checkout 0
            }
        }
    }
}

/// Handle quantity selection for an item
fn handle_quantity_selection(
    index: u8,
    item_id: &str,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    if index == 0 {
        handle_back_navigation(state, store_state);
        return;
    }

    if let Some(item) = state.store_data.find_item(item_id) {
        let mut new_state = state.clone();

        match index {
            1 => {
                // Add +1
                if can_add_item(
                    &new_state.cart,
                    item,
                    1,
                    props.game_state.budget_cents,
                    state.discount_pct,
                ) {
                    new_state.cart.add_item(item_id, 1);
                    announce_quantity_change(
                        item,
                        1,
                        true,
                        &new_state,
                        props.game_state.budget_cents,
                    );
                } else {
                    announce_cannot_add(item);
                }
            }
            2 => {
                // Add +5 (if not unique)
                if !item.unique
                    && can_add_item(
                        &new_state.cart,
                        item,
                        5,
                        props.game_state.budget_cents,
                        state.discount_pct,
                    )
                {
                    new_state.cart.add_item(item_id, 5);
                    announce_quantity_change(
                        item,
                        5,
                        true,
                        &new_state,
                        props.game_state.budget_cents,
                    );
                } else {
                    announce_cannot_add(item);
                }
            }
            3 => {
                // Remove -1
                if new_state.cart.get_quantity(item_id) > 0 {
                    new_state.cart.remove_item(item_id, 1);
                    announce_quantity_change(
                        item,
                        1,
                        false,
                        &new_state,
                        props.game_state.budget_cents,
                    );
                }
            }
            4 => {
                // Remove all
                if new_state.cart.get_quantity(item_id) > 0 {
                    new_state.cart.remove_all_item(item_id);
                    announce_quantity_change(
                        item,
                        0,
                        false,
                        &new_state,
                        props.game_state.budget_cents,
                    );
                }
            }
            _ => return,
        }

        // Update cart total
        new_state.cart.total_cents =
            calculate_cart_total(&new_state.cart, &state.store_data, state.discount_pct);
        store_state.set(new_state);
    }
}

/// Check if an item can be added to the cart
fn can_add_item(
    cart: &Cart,
    item: &StoreItem,
    qty_to_add: i32,
    budget_cents: i64,
    discount_pct: f64,
) -> bool {
    let current_qty = cart.get_quantity(&item.id);
    let new_qty = current_qty + qty_to_add;

    // Check max quantity
    if new_qty > item.max_qty {
        return false;
    }

    // Check unique constraint
    if item.unique && new_qty > 1 {
        return false;
    }

    // Check budget
    let effective_price = calculate_effective_price(item.price_cents, discount_pct);
    let additional_cost = effective_price * i64::from(qty_to_add);
    let new_total = cart.total_cents + additional_cost;

    new_total <= budget_cents
}

/// Announce quantity changes to screen readers
fn announce_quantity_change(
    item: &StoreItem,
    qty: i32,
    added: bool,
    state: &StoreState,
    budget_cents: i64,
) {
    let item_name = i18n::t(&format!("store.items.{}.name", item.id));
    let effective_price = calculate_effective_price(item.price_cents, state.discount_pct);
    let price_str = format_currency(if added {
        effective_price * i64::from(qty)
    } else {
        effective_price
    });
    let remaining = budget_cents - state.cart.total_cents;
    let remaining_str = format_currency(remaining);

    let message = if added {
        i18n::tr(
            "store.alerts.added",
            Some(&{
                let mut vars = HashMap::new();
                vars.insert("item", item_name.as_str());
                vars.insert("price", price_str.as_str());
                vars.insert("left", remaining_str.as_str());
                vars
            }),
        )
    } else {
        i18n::tr(
            "store.alerts.removed",
            Some(&{
                let mut vars = HashMap::new();
                vars.insert("item", item_name.as_str());
                vars.insert("left", remaining_str.as_str());
                vars
            }),
        )
    };

    announce_to_screen_reader(&message);
}

/// Announce when item cannot be added
fn announce_cannot_add(item: &StoreItem) {
    let item_name = i18n::t(&format!("store.items.{}.name", item.id));
    let message = if item.unique {
        i18n::tr(
            "store.alerts.unique",
            Some(&{
                let mut vars = HashMap::new();
                vars.insert("item", item_name.as_str());
                vars
            }),
        )
    } else {
        i18n::tr(
            "store.alerts.max_qty",
            Some(&{
                let mut vars = HashMap::new();
                vars.insert("item", item_name.as_str());
                vars
            }),
        )
    };

    announce_to_screen_reader(&message);
}

/// Handle cart screen selections
fn handle_cart_selection(
    index: u8,
    state: &StoreState,
    store_state: &UseStateHandle<StoreState>,
    props: &OutfittingStoreProps,
) {
    if index == 0 {
        // Checkout
        let remaining_budget = props.game_state.budget_cents - state.cart.total_cents;
        if remaining_budget >= 0 {
            handle_checkout(state, props);
        } else {
            announce_to_screen_reader(&i18n::t("store.alerts.over_budget"));
        }
    } else if let Some(cart_line) = state.cart.lines.get((index - 1) as usize) {
        // Open quantity prompt for this cart item
        let mut new_state = state.clone();
        new_state.current_screen = StoreScreen::QuantityPrompt(cart_line.item_id.clone());
        new_state.focus_idx = 1;
        store_state.set(new_state);
    }
}

/// Handle final checkout
fn handle_checkout(state: &StoreState, props: &OutfittingStoreProps) {
    let mut total_grants = Grants::default();
    let mut all_tags = Vec::new();

    // Aggregate all grants and tags from cart
    for line in &state.cart.lines {
        if let Some(item) = state.store_data.find_item(&line.item_id) {
            total_grants.supplies += item.grants.supplies * line.qty;
            total_grants.credibility += item.grants.credibility * line.qty;
            total_grants.spare_tire += item.grants.spare_tire * line.qty;
            total_grants.spare_battery += item.grants.spare_battery * line.qty;
            total_grants.spare_alt += item.grants.spare_alt * line.qty;
            total_grants.spare_pump += item.grants.spare_pump * line.qty;

            // Add tags (will be deduplicated by GameState)
            for tag in &item.tags {
                all_tags.push(tag.clone());
            }
        }
    }

    // Create updated game state
    let mut new_game_state = props.game_state.clone();
    new_game_state.apply_store_purchase(state.cart.total_cents, &total_grants, &all_tags);

    // Emit the continue event
    props
        .on_continue
        .emit((new_game_state, total_grants, all_tags));
}

/// Format currency using Intl API
fn format_currency(cents: i64) -> String {
    #[allow(clippy::cast_precision_loss)]
    let dollars = cents as f64 / 100.0;

    // Use the browser's Intl API for currency formatting
    let Some(window) = window() else {
        return format!("${dollars:.2}");
    };
    let Ok(intl) = js_sys::Reflect::get(&window, &"Intl".into()) else {
        return format!("${dollars:.2}");
    };
    let Ok(number_format) = js_sys::Reflect::get(&intl, &"NumberFormat".into()) else {
        return format!("${dollars:.2}");
    };

    let locale = "en-US".into();
    let options = {
        let options = js_sys::Object::new();
        // Set currency formatting options, ignore errors for fallback behavior
        let _ = js_sys::Reflect::set(&options, &"style".into(), &"currency".into());
        let _ = js_sys::Reflect::set(&options, &"currency".into(), &"USD".into());
        options
    };
    let args = js_sys::Array::of2(&locale, &options.into());
    let Ok(formatter) = js_sys::Reflect::construct(&number_format.into(), &args) else {
        return format!("${dollars:.2}");
    };
    let Ok(format_fn) = js_sys::Reflect::get(&formatter, &"format".into()) else {
        return format!("${dollars:.2}");
    };
    let Ok(result) = js_sys::Reflect::apply(
        &format_fn.into(),
        &formatter,
        &js_sys::Array::of1(&dollars.into()),
    ) else {
        return format!("${dollars:.2}");
    };
    if let Some(formatted) = result.as_string() {
        return formatted;
    }

    // Fallback formatting
    format!("${dollars:.2}")
}

/// Announce message to screen readers
fn announce_to_screen_reader(message: &str) {
    set_status(message);
}

/// Render the main store screen
fn render_home_screen(
    state: &UseStateHandle<StoreState>,
    game_state: &GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<KeyboardEvent>,
) -> Html {
    let budget_str = format_currency(game_state.budget_cents - state.cart.total_cents);
    let title = i18n::tr(
        "store.menu.home",
        Some(&{
            let mut vars = HashMap::new();
            vars.insert("budget", budget_str.as_str());
            vars
        }),
    );

    let categories = [
        (1u8, i18n::t("store.categories.fuel_food")),
        (2u8, i18n::t("store.categories.vehicle")),
        (3u8, i18n::t("store.categories.ppe")),
        (4u8, i18n::t("store.categories.docs")),
        (5u8, i18n::t("store.menu.view_cart")),
        (0u8, i18n::t("store.menu.continue")),
    ];

    let remaining_budget = game_state.budget_cents - state.cart.total_cents;
    let can_continue = remaining_budget >= 0;
    let budget_class = if remaining_budget < 0 {
        "budget over"
    } else {
        "budget ok"
    };

    let on_tab = |category_id: &str| {
        let state = state.clone();
        let cat_id = category_id.to_string();
        Callback::from(move |_| set_screen(&state, StoreScreen::Category(cat_id.clone())))
    };

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="store-title" onkeydown={on_keydown} class="store-shell">
                <header class="store-header">
                    <div>
                        <h1 id="store-title">{ title.clone() }</h1>
                    </div>
                    <div class={classes!("store-budget", budget_class)}>
                        <span class="label">{ i18n::t("store.budget") }</span>
                        <span class="value">{ budget_str.clone() }</span>
                    </div>
                </header>
                <nav class="store-tabs" aria-label={i18n::t("store.title")}>
                    {
                        state.store_data.categories.iter().enumerate().map(|(i, cat)| {
                            let idx = u8::try_from(i + 1).unwrap_or(0);
                            let focused = state.focus_idx == idx;
                            html! {
                                <button
                                    class="store-tab"
                                    data-key={idx.to_string()}
                                    aria-pressed="false"
                                    tabindex={if focused { "0" } else { "-1" }}
                                    onclick={on_tab(&cat.id)}>
                                    { i18n::t(&format!("store.categories.{}", cat.id)) }
                                </button>
                            }
                        }).collect::<Html>()
                    }
                </nav>
                <ul role="menu" aria-label={i18n::t("store.title")} ref={list_ref} class="store-menu">
                    { for categories.iter().enumerate().map(|(i, (idx, label))| {
                        let focused = state.focus_idx == *idx;
                        let disabled = *idx == 0 && !can_continue;
                        let posinset = u8::try_from(i).unwrap_or(0) + 1;

                        html!{
                            <li role="menuitem"
                                tabindex={if focused && !disabled { "0" } else { "-1" }}
                                data-key={idx.to_string()}
                                aria-posinset={posinset.to_string()}
                                aria-setsize="6"
                                aria-disabled={disabled.to_string()}
                                class={classes!("ot-menuitem", disabled.then_some("disabled"))}>
                                <span class="num">{ format!("{})", idx) }</span>
                                <span class="label">{ label.clone() }</span>
                            </li>
                        }
                    }) }
                </ul>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}

/// Render category screen
fn render_category_screen(
    category_id: &str,
    state: &UseStateHandle<StoreState>,
    game_state: &GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<KeyboardEvent>,
) -> Html {
    let Some(category) = state
        .store_data
        .categories
        .iter()
        .find(|c| c.id == *category_id)
    else {
        return html! { <div>{ "Category not found" }</div> };
    };

    let budget_str = format_currency(game_state.budget_cents - state.cart.total_cents);
    let category_name = i18n::t(&format!("store.categories.{category_id}"));
    let title = format!(
        "{category_name} — {budget_label}: {budget_str}",
        budget_label = i18n::t("store.budget")
    );

    let items = category.items.clone();
    let on_nav = {
        let state = state.clone();
        Callback::from(move |_| set_screen(&state, StoreScreen::Home))
    };

    let cart_cta = Callback::from({
        let state = state.clone();
        move |_| set_screen(&state, StoreScreen::Cart)
    });

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="category-title" onkeydown={on_keydown} class="store-shell">
                <header class="store-header">
                    <div>
                        <h1 id="category-title">{ title }</h1>
                    </div>
                    <div class="store-budget">
                        <span class="label">{ i18n::t("store.budget") }</span>
                        <span class="value">{ budget_str }</span>
                    </div>
                </header>
                <div class="store-cart-summary" role="status" aria-live="polite">
                    <span>{ i18n::t("store.cart.title") }</span>
                    <span class="value">{ format_currency(state.cart.total_cents) }</span>
                </div>
                <div class="store-item-grid" ref={list_ref}>
                    { for items.iter().enumerate().map(|(i, item)| render_store_item_card(
                        u8::try_from(i + 1).unwrap_or(0),
                        item,
                        state,
                        game_state
                    )) }
                </div>
                <div class="store-footer-row">
                    <button class="retro-btn-secondary" onclick={on_nav}>{ i18n::t("store.menu.back") }</button>
                    <button class="retro-btn-primary" onclick={cart_cta}>{ i18n::t("store.menu.view_cart") }</button>
                </div>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}

fn render_store_item_card(
    idx: u8,
    item: &StoreItem,
    state: &UseStateHandle<StoreState>,
    game_state: &GameState,
) -> Html {
    let name = i18n::t(&format!("store.items.{}.name", item.id));
    let desc = i18n::t(&format!("store.items.{}.desc", item.id));
    let effective_price = calculate_effective_price(item.price_cents, state.discount_pct);
    let price_str = format_currency(effective_price);
    let qty_in_cart = state.cart.get_quantity(&item.id);
    let can_add = can_add_item(
        &state.cart,
        item,
        1,
        game_state.budget_cents,
        state.discount_pct,
    );
    let initials = name
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>());

    let on_add = {
        let state = state.clone();
        let item_clone = item.clone();
        let budget = game_state.budget_cents;
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            if can_add_item(
                &new_state.cart,
                &item_clone,
                1,
                budget,
                new_state.discount_pct,
            ) {
                new_state.cart.add_item(&item_clone.id, 1);
                announce_quantity_change(&item_clone, 1, true, &new_state, budget);
                new_state.cart.total_cents = calculate_cart_total(
                    &new_state.cart,
                    &new_state.store_data,
                    new_state.discount_pct,
                );
                new_state.focus_idx = idx;
                state.set(new_state);
            }
        })
    };

    let on_remove = {
        let state = state.clone();
        let item_clone = item.clone();
        let budget = game_state.budget_cents;
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            if new_state.cart.get_quantity(&item_clone.id) > 0 {
                new_state.cart.remove_item(&item_clone.id, 1);
                announce_quantity_change(&item_clone, 1, false, &new_state, budget);
                new_state.cart.total_cents = calculate_cart_total(
                    &new_state.cart,
                    &new_state.store_data,
                    new_state.discount_pct,
                );
                new_state.focus_idx = idx;
                state.set(new_state);
            }
        })
    };

    html! {
        <article
            role="group"
            aria-labelledby={format!("store-item-{idx}")}
            class="store-card"
            data-key={idx.to_string()}
            title={desc.clone()}>
            <div class="store-card-icon" aria-hidden="true">
                <span>{ initials }</span>
            </div>
            <div class="store-card-body">
                <div class="store-card-head">
                    <h2 id={format!("store-item-{idx}")}>{ name }</h2>
                    <span class="store-price">{ price_str }</span>
                </div>
                <p class="muted">{ desc }</p>
                <div class="store-qty-row">
                    <button class="store-qty-btn" onclick={on_remove} aria-label={i18n::t("store.qty_prompt.rem1")} disabled={qty_in_cart == 0}>{"–"}</button>
                    <span class="store-qty" aria-live="polite">{ qty_in_cart }</span>
                    <button class="store-qty-btn" onclick={on_add} aria-label={i18n::t("store.qty_prompt.add1")} disabled={!can_add}>{"+"}</button>
                </div>
            </div>
        </article>
    }
}

/// Render quantity prompt screen
#[allow(clippy::too_many_lines)]
fn render_quantity_screen(
    item_id: &str,
    state: &UseStateHandle<StoreState>,
    game_state: &GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<KeyboardEvent>,
) -> Html {
    let Some(item) = state.store_data.find_item(item_id) else {
        return html! { <div>{ "Item not found" }</div> };
    };

    let item_name = i18n::t(&format!("store.items.{}.name", item.id));
    let effective_price = calculate_effective_price(item.price_cents, state.discount_pct);
    let price_str = format_currency(effective_price);

    let title = i18n::tr(
        "store.qty_prompt.title",
        Some(&{
            let mut vars = HashMap::new();
            vars.insert("item", item_name.as_str());
            vars.insert("price", price_str.as_str());
            vars
        }),
    );

    let current_qty = state.cart.get_quantity(item_id);
    let mut options = Vec::new();

    // Add +1 option
    let can_add_1 = can_add_item(
        &state.cart,
        item,
        1,
        game_state.budget_cents,
        state.discount_pct,
    );
    if can_add_1 {
        let budget_preview = game_state.budget_cents - state.cart.total_cents - effective_price;
        let preview_str = format!(
            "[{}{}]",
            i18n::t("store.budget"),
            format_currency(budget_preview)
        );
        options.push((1u8, i18n::t("store.qty_prompt.add1"), preview_str));
    } else {
        options.push((
            1u8,
            i18n::t("store.qty_prompt.add1"),
            String::from("[Max/Budget]"),
        ));
    }

    // Add +5 option (if not unique)
    if !item.unique {
        let can_add_5 = can_add_item(
            &state.cart,
            item,
            5,
            game_state.budget_cents,
            state.discount_pct,
        );
        if can_add_5 {
            let budget_preview =
                game_state.budget_cents - state.cart.total_cents - (effective_price * 5);
            let preview_str = format!(
                "[{}{}]",
                i18n::t("store.budget"),
                format_currency(budget_preview)
            );
            options.push((2u8, i18n::t("store.qty_prompt.add5"), preview_str));
        } else {
            options.push((
                2u8,
                i18n::t("store.qty_prompt.add5"),
                String::from("[Max/Budget]"),
            ));
        }
    }

    // Remove -1 option (if we have any)
    if current_qty > 0 {
        options.push((3u8, i18n::t("store.qty_prompt.rem1"), String::new()));
    }

    // Remove all option (if we have any)
    if current_qty > 0 {
        options.push((4u8, i18n::t("store.qty_prompt.rem_all"), String::new()));
    }

    // Back option
    options.push((0u8, i18n::t("store.menu.back"), String::new()));

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="qty-title" onkeydown={on_keydown}>
                <h1 id="qty-title">{ title }</h1>
                { if current_qty > 0 {
                    html! { <p class="current-qty">{ format!("Current in cart: {current_qty}") }</p> }
                } else {
                    html! {}
                }}
                <ul role="menu" aria-label={item_name} ref={list_ref}>
                    { for options.iter().enumerate().map(|(i, (idx, label, preview))| {
                        let focused = state.focus_idx == *idx;
                        let posinset = u8::try_from(i).unwrap_or(0) + 1;

                        html!{
                            <li role="menuitem"
                                tabindex={if focused { "0" } else { "-1" }}
                                data-key={idx.to_string()}
                                aria-posinset={posinset.to_string()}
                                aria-setsize={options.len().to_string()}
                                class="ot-menuitem">
                                <span class="num">{ format!("{})", idx) }</span>
                                <span class="label">
                                    { label.clone() }
                                    { if preview.is_empty() {
                                        html! {}
                                    } else {
                                        html! { <span class="preview">{ format!(" {preview}") }</span> }
                                    }}
                                </span>
                            </li>
                        }
                    }) }
                </ul>
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}

/// Render cart screen
fn render_cart_screen(
    state: &UseStateHandle<StoreState>,
    game_state: &GameState,
    list_ref: &NodeRef,
    on_keydown: &Callback<KeyboardEvent>,
) -> Html {
    let total_str = format_currency(state.cart.total_cents);
    let remaining = game_state.budget_cents - state.cart.total_cents;
    let remaining_str = format_currency(remaining);

    let cart_total_line = i18n::tr(
        "store.cart.total",
        Some(&{
            let mut vars = HashMap::new();
            vars.insert("sum", total_str.as_str());
            vars.insert("left", remaining_str.as_str());
            vars
        }),
    );

    let can_checkout = remaining >= 0;
    let mut cart_lines = Vec::new();

    // Add cart items
    for (i, line) in state.cart.lines.iter().enumerate() {
        if let Some(item) = state.store_data.find_item(&line.item_id) {
            let idx = u8::try_from(i + 1).unwrap_or(255);
            let item_name = i18n::t(&format!("store.items.{}.name", item.id));
            let effective_price = calculate_effective_price(item.price_cents, state.discount_pct);
            let line_total = effective_price * i64::from(line.qty);
            let line_total_str = format_currency(line_total);

            let qty_str = line.qty.to_string();
            let label = i18n::tr(
                "store.cart.line",
                Some(&{
                    let mut vars = HashMap::new();
                    vars.insert("item", item_name.as_str());
                    vars.insert("qty", qty_str.as_str());
                    vars.insert("line_total", line_total_str.as_str());
                    vars
                }),
            );

            cart_lines.push((idx, label));
        }
    }

    // Add checkout option
    cart_lines.push((0u8, i18n::t("store.cart.checkout")));

    html! {
        <main class="outfitting-store">
            <section role="region" aria-labelledby="cart-title" onkeydown={on_keydown}>
                <h1 id="cart-title">{ i18n::t("store.cart.title") }</h1>
                { if state.cart.lines.is_empty() {
                    html! { <p class="empty-cart">{ "Your cart is empty." }</p> }
                } else {
                    html! {
                        <>
                            <ul role="menu" aria-label={i18n::t("store.cart.title")} ref={list_ref}>
                                { for cart_lines.iter().enumerate().map(|(i, (idx, label))| {
                                    let focused = state.focus_idx == *idx;
                                    let disabled = *idx == 0 && !can_checkout;
                                    let posinset = u8::try_from(i).unwrap_or(0) + 1;

                                    html!{
                                        <li role="menuitem"
                                            tabindex={if focused && !disabled { "0" } else { "-1" }}
                                            data-key={idx.to_string()}
                                            aria-posinset={posinset.to_string()}
                                            aria-setsize={cart_lines.len().to_string()}
                                            aria-disabled={disabled.to_string()}
                                            class={classes!("ot-menuitem", disabled.then_some("disabled"))}>
                                            <span class="num">{ format!("{})", idx) }</span>
                                            <span class="label">{ label.clone() }</span>
                                        </li>
                                    }
                                }) }
                            </ul>
                            <p class="cart-total" aria-live="polite">{ cart_total_line }</p>
                            { if can_checkout {
                                html! {}
                            } else {
                                html! { <p class="error" role="alert">{ i18n::t("store.alerts.over_budget") }</p> }
                            }}
                        </>
                    }
                }}
                <div aria-live="polite" aria-atomic="true" class="sr-only" id="store-status"></div>
            </section>
        </main>
    }
}
