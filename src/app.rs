use yew::prelude::*;
mod components { pub mod header; pub mod footer; pub mod button; pub mod modal; pub mod form { pub mod field; pub mod text_input; } }
mod pages { pub mod home; pub mod settings; }

#[function_component(App)]
pub fn app() -> Html {
    html!{ <pages::home::Home /> }
}
