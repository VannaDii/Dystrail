use yew_router::prelude::*;

#[derive(Clone, Debug, Routable, PartialEq, Eq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/persona")]
    Persona,
    #[at("/outfitting")]
    Outfitting,
    #[at("/game")]
    Game,
    #[at("/travel")]
    Travel,
    #[at("/store")]
    Store,
    #[at("/crossing")]
    Crossing,
    #[at("/route")]
    RoutePrompt,
    #[at("/camp")]
    Camp,
    #[at("/encounter")]
    Encounter,
    #[at("/boss")]
    Boss,
    #[at("/result")]
    Result,
    #[at("/404")]
    #[not_found]
    NotFound,
}

impl Route {
    #[must_use]
    pub const fn from_phase(phase: &crate::app::Phase) -> Self {
        match phase {
            crate::app::Phase::Persona => Self::Persona,
            crate::app::Phase::Outfitting => Self::Outfitting,
            crate::app::Phase::Menu | crate::app::Phase::Boot => Self::Home,
            crate::app::Phase::Travel => Self::Travel,
            crate::app::Phase::Store => Self::Store,
            crate::app::Phase::Crossing => Self::Crossing,
            crate::app::Phase::RoutePrompt => Self::RoutePrompt,
            crate::app::Phase::Camp => Self::Camp,
            crate::app::Phase::Encounter => Self::Encounter,
            crate::app::Phase::Boss => Self::Boss,
            crate::app::Phase::Result => Self::Result,
        }
    }

    #[must_use]
    pub const fn to_phase(&self) -> Option<crate::app::Phase> {
        match self {
            Self::Persona => Some(crate::app::Phase::Persona),
            Self::Outfitting => Some(crate::app::Phase::Outfitting),
            Self::Home | Self::NotFound => None, // Preserve current phase on Home / 404 routes.
            Self::Game => Some(crate::app::Phase::Menu),
            Self::Travel => Some(crate::app::Phase::Travel),
            Self::Store => Some(crate::app::Phase::Store),
            Self::Crossing => Some(crate::app::Phase::Crossing),
            Self::RoutePrompt => Some(crate::app::Phase::RoutePrompt),
            Self::Camp => Some(crate::app::Phase::Camp),
            Self::Encounter => Some(crate::app::Phase::Encounter),
            Self::Boss => Some(crate::app::Phase::Boss),
            Self::Result => Some(crate::app::Phase::Result),
        }
    }
}
