use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Eq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/game")]
    Game,
    #[at("/travel")]
    Travel,
    #[at("/encounter")]
    Encounter,
    #[at("/boss")]
    Boss,
    #[at("/result")]
    Result,
}

impl Route {
    #[must_use]
    pub fn from_phase(phase: &crate::app::Phase) -> Self {
        match phase {
            crate::app::Phase::Menu | crate::app::Phase::Boot => Route::Home,
            crate::app::Phase::Travel => Route::Travel,
            crate::app::Phase::Encounter => Route::Encounter,
            crate::app::Phase::Boss => Route::Boss,
            crate::app::Phase::Result => Route::Result,
        }
    }

    #[must_use]
    pub fn to_phase(&self) -> Option<crate::app::Phase> {
        match self {
            Route::Game | Route::Home => Some(crate::app::Phase::Menu),
            Route::Travel => Some(crate::app::Phase::Travel),
            Route::Encounter => Some(crate::app::Phase::Encounter),
            Route::Boss => Some(crate::app::Phase::Boss),
            Route::Result => Some(crate::app::Phase::Result),
        }
    }
}
