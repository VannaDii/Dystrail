use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Eq)]
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
    #[at("/camp")]
    Camp,
    #[at("/encounter")]
    Encounter,
    #[at("/boss")]
    Boss,
    #[at("/result")]
    Result,
}

impl Route {
    #[must_use]
    pub const fn from_phase(phase: &crate::app::Phase) -> Self {
        match phase {
            crate::app::Phase::Persona => Self::Persona,
            crate::app::Phase::Outfitting => Self::Outfitting,
            crate::app::Phase::Menu | crate::app::Phase::Boot => Self::Home,
            crate::app::Phase::Travel => Self::Travel,
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
            Self::Home | Self::Game => Some(crate::app::Phase::Menu),
            Self::Travel => Some(crate::app::Phase::Travel),
            Self::Camp => Some(crate::app::Phase::Camp),
            Self::Encounter => Some(crate::app::Phase::Encounter),
            Self::Boss => Some(crate::app::Phase::Boss),
            Self::Result => Some(crate::app::Phase::Result),
        }
    }
}
