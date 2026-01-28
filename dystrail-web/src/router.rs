use yew_router::prelude::*;

#[derive(Clone, Debug, Routable, PartialEq, Eq)]
pub enum Route {
    #[at("/")]
    Boot,
    #[at("/menu")]
    Menu,
    #[at("/about")]
    About,
    #[at("/settings")]
    Settings,
    #[at("/persona")]
    Persona,
    #[at("/mode")]
    ModeSelect,
    #[at("/outfitting")]
    Outfitting,
    #[at("/travel")]
    Travel,
    #[at("/inventory")]
    Inventory,
    #[at("/pace-diet")]
    PaceDiet,
    #[at("/map")]
    Map,
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
            crate::app::Phase::Boot => Self::Boot,
            crate::app::Phase::Menu => Self::Menu,
            crate::app::Phase::About => Self::About,
            crate::app::Phase::Settings => Self::Settings,
            crate::app::Phase::Persona => Self::Persona,
            crate::app::Phase::ModeSelect => Self::ModeSelect,
            crate::app::Phase::Outfitting => Self::Outfitting,
            crate::app::Phase::Travel => Self::Travel,
            crate::app::Phase::Inventory => Self::Inventory,
            crate::app::Phase::PaceDiet => Self::PaceDiet,
            crate::app::Phase::Map => Self::Map,
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
            Self::Boot => Some(crate::app::Phase::Boot),
            Self::Menu => Some(crate::app::Phase::Menu),
            Self::About => Some(crate::app::Phase::About),
            Self::Settings => Some(crate::app::Phase::Settings),
            Self::Persona => Some(crate::app::Phase::Persona),
            Self::ModeSelect => Some(crate::app::Phase::ModeSelect),
            Self::Outfitting => Some(crate::app::Phase::Outfitting),
            Self::Travel => Some(crate::app::Phase::Travel),
            Self::Inventory => Some(crate::app::Phase::Inventory),
            Self::PaceDiet => Some(crate::app::Phase::PaceDiet),
            Self::Map => Some(crate::app::Phase::Map),
            Self::Store => Some(crate::app::Phase::Store),
            Self::Crossing => Some(crate::app::Phase::Crossing),
            Self::RoutePrompt => Some(crate::app::Phase::RoutePrompt),
            Self::Camp => Some(crate::app::Phase::Camp),
            Self::Encounter => Some(crate::app::Phase::Encounter),
            Self::Boss => Some(crate::app::Phase::Boss),
            Self::Result => Some(crate::app::Phase::Result),
            Self::NotFound => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_route_maps_to_none() {
        assert_eq!(Route::NotFound.to_phase(), None);
    }
}
