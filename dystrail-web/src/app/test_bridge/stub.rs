use crate::app::state::AppState;
use yew::prelude::*;

#[hook]
pub fn use_test_bridge(app_state: &AppState) {
    let _ = app_state;
}

#[cfg(test)]
mod tests {
    use crate::app::state::use_app_state;
    use crate::app::test_bridge::use_test_bridge;
    use futures::executor::block_on;
    use yew::LocalServerRenderer;
    use yew::prelude::*;

    #[function_component(TestBridgeHarness)]
    fn test_bridge_harness() -> Html {
        let state = use_app_state();
        use_test_bridge(&state);
        html! { <span>{ "ok" }</span> }
    }

    #[test]
    fn test_bridge_stub_renders() {
        let html = block_on(LocalServerRenderer::<TestBridgeHarness>::new().render());
        assert!(html.contains("ok"));
    }
}
