use yew::prelude::*;

#[function_component(Modal)]
pub fn modal() -> Html {
    html! {
        <div role="dialog" aria-modal="true">{"Modal"}</div>
    }
}
