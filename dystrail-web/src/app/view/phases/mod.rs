pub mod aftermath;
mod boss;
mod camp;
mod crew;
mod encounter;
mod menu;
mod outfitting;
mod persona;
mod result;
mod travel;

use crate::app::phase::Phase;
use crate::app::state::AppState;
use crate::app::view::handlers::AppHandlers;
use crate::pages::not_found::NotFound;
use crate::router::Route;
use yew::prelude::*;

pub use boss::render_boss;
pub use camp::render_camp;
pub use encounter::render_encounter;
pub use menu::render_menu;
pub use outfitting::render_outfitting;
pub use persona::render_persona;
pub use result::render_result;
pub use travel::render_travel;

pub fn render_main_view(state: &AppState, handlers: &AppHandlers, route: Option<&Route>) -> Html {
    let not_found = matches!(route, None | Some(Route::NotFound));
    if not_found {
        return html! { <NotFound on_go_home={handlers.go_home.clone()} /> };
    }

    if *state.town_open {
        return crate::app::services::render_shop(state);
    }
    if *state.phase == Phase::Map && *state.map_automatic {
        return html! {<>
            {crate::app::map::render(state)}
            {state.pending_turn.as_ref().map(|pending|crate::app::turn::render_transit(state,pending)).unwrap_or_default()}
        </>};
    }
    if state.pending_turn.is_some() {
        return render_travel(state, handlers);
    }
    if let Some(feedback) = state.aftermath.as_ref() {
        return aftermath::render_aftermath(state, feedback);
    }

    match *state.phase {
        Phase::Boot => {
            let boot_logo_src: AttrValue = crate::paths::asset_path("static/img/logo.png").into();
            html! {
                <crate::pages::boot::BootPage
                    code={(*state.code).clone()}
                    on_code_change={{let code = state.code.clone(); Callback::from(move |value| code.set(value))}}
                    logo_src={boot_logo_src}
                    ready={*state.boot_ready}
                    preload_progress={*state.preload_progress}
                    on_begin={handlers.begin_boot.clone()}
                />
            }
        }
        Phase::Persona => render_persona(state),
        Phase::Crew => crew::render_crew(state),
        Phase::Outfitting => render_outfitting(state),
        Phase::Menu => render_menu(state),
        Phase::Town => crate::app::town::render(state),
        Phase::CrewCare => crate::app::crew_care::render(state),
        Phase::AllyLoss => crate::app::ally_loss::render(state),
        Phase::Travel => render_travel(state, handlers),
        Phase::Map => crate::app::map::render(state),
        Phase::Camp => render_camp(state),
        Phase::Encounter => render_encounter(state, handlers),
        Phase::Boss => render_boss(state, handlers),
        Phase::Result => render_result(state),
    }
}
