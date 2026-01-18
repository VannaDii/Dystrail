mod boss;
mod camp;
mod crossing;
mod encounter;
mod menu;
mod outfitting;
mod persona;
mod result;
mod route_prompt;
mod seed_footer;
mod store;
mod travel;

use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::not_found::NotFound;
use crate::router::Route;
use yew::prelude::*;

pub use boss::render_boss;
pub use camp::render_camp;
pub use crossing::render_crossing;
pub use encounter::render_encounter;
pub use menu::render_menu;
pub use outfitting::render_outfitting;
pub use persona::render_persona;
pub use result::render_result;
pub use route_prompt::render_route_prompt;
pub use seed_footer::render_seed_footer;
pub use store::render_store;
pub use travel::render_travel;

pub fn render_main_view(state: &AppState, handlers: &AppHandlers, route: Option<&Route>) -> Html {
    let not_found = matches!(route, None | Some(Route::NotFound));
    if not_found {
        return html! { <NotFound on_go_home={handlers.go_home.clone()} /> };
    }

    match *state.phase {
        Phase::Boot => {
            let boot_logo_src: AttrValue = crate::paths::asset_path("static/img/logo.png").into();
            html! {
                <crate::pages::boot::BootPage
                    logo_src={boot_logo_src}
                    ready={*state.boot_ready}
                    preload_progress={*state.preload_progress}
                    on_begin={handlers.begin_boot.clone()}
                />
            }
        }
        Phase::Persona => render_persona(state),
        Phase::Outfitting => render_outfitting(state),
        Phase::Menu => render_menu(state),
        Phase::Travel => render_travel(state, handlers),
        Phase::Store => render_store(state, handlers),
        Phase::Crossing => render_crossing(state, handlers),
        Phase::RoutePrompt => render_route_prompt(state, handlers),
        Phase::Camp => render_camp(state),
        Phase::Encounter => render_encounter(state, handlers),
        Phase::Boss => render_boss(state, handlers),
        Phase::Result => render_result(state),
    }
}
