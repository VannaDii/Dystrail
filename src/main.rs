#![forbid(unsafe_code)]
use yew::prelude::*;
mod app;
fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    yew::Renderer::<app::App>::new().render();
}
