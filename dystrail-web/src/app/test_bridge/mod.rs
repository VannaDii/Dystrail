#[cfg(any(test, target_arch = "wasm32"))]
mod shared;
#[cfg(not(target_arch = "wasm32"))]
mod stub;
#[cfg(target_arch = "wasm32")]
mod wasm;

use crate::app::state::AppState;
use yew::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
#[hook]
pub fn use_test_bridge(app_state: &AppState) {
    stub::use_test_bridge(app_state);
}

#[cfg(target_arch = "wasm32")]
#[hook]
pub fn use_test_bridge(app_state: &AppState) {
    wasm::use_test_bridge(app_state);
}
