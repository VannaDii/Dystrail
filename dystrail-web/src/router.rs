use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Eq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/persona")]
    Persona,
    #[at("/crew")]
    Crew,
    #[at("/outfitting")]
    Outfitting,
    #[at("/game")]
    Game,
    #[at("/map")]
    Map,
    #[at("/travel")]
    Travel,
    #[at("/town")]
    Town,
    #[at("/crew-care")]
    CrewCare,
    #[at("/ally-loss")]
    AllyLoss,
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
            crate::app::Phase::Crew => Self::Crew,
            crate::app::Phase::Outfitting => Self::Outfitting,
            crate::app::Phase::Menu | crate::app::Phase::Boot => Self::Home,
            crate::app::Phase::Travel => Self::Travel,
            crate::app::Phase::Town => Self::Town,
            crate::app::Phase::CrewCare => Self::CrewCare,
            crate::app::Phase::AllyLoss => Self::AllyLoss,
            crate::app::Phase::Map => Self::Map,
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
            Self::Crew => Some(crate::app::Phase::Crew),
            Self::Outfitting => Some(crate::app::Phase::Outfitting),
            Self::Home | Self::NotFound => None, // Preserve current phase on Home / 404 routes.
            Self::Game => Some(crate::app::Phase::Menu),
            Self::Travel => Some(crate::app::Phase::Travel),
            Self::Town => Some(crate::app::Phase::Town),
            Self::CrewCare => Some(crate::app::Phase::CrewCare),
            Self::AllyLoss => Some(crate::app::Phase::AllyLoss),
            Self::Map => Some(crate::app::Phase::Map),
            Self::Camp => Some(crate::app::Phase::Camp),
            Self::Encounter => Some(crate::app::Phase::Encounter),
            Self::Boss => Some(crate::app::Phase::Boss),
            Self::Result => Some(crate::app::Phase::Result),
        }
    }
}
