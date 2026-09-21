//! Versioned crew artwork. Stable role IDs select identities; poses never change game state.
use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    Standard,
    Happy,
    Defeated,
    Unwell,
    Standing,
    PassengerNear,
    PassengerFar,
    Driver,
}

impl Pose {
    #[must_use]
    pub const fn cell(self) -> u8 {
        match self {
            Self::Standard => 0,
            Self::Happy => 1,
            Self::Defeated => 2,
            Self::Unwell => 3,
            Self::Standing => 4,
            Self::PassengerNear => 5,
            Self::PassengerFar => 6,
            Self::Driver => 7,
        }
    }
}

#[must_use]
pub fn path(persona: &str) -> String {
    let role = match persona {
        "organizer" | "whistleblower" | "lobbyist" | "staffer" | "satirist" => persona,
        _ => "journalist",
    };
    format!("static/img/cast-v2/{role}.png")
}

/// Logical sheet coordinates decouple rendering from the generated image's pixel size.
/// A small inset keeps neighboring cells and the portrait baseline outside the crop.
#[must_use]
pub fn view_box(pose: Pose) -> String {
    let cell = u32::from(pose.cell());
    let x = (cell % 4) * 512;
    let y = (cell / 4) * 512;
    format!("{x} {y} 512 {}", if cell < 4 { 486 } else { 512 })
}

pub fn art(persona: &str, pose: Pose) -> Html {
    html! {<svg class="cast-art" data-cast-version="2" data-pose={pose.cell().to_string()}
        viewBox={view_box(pose)} aria-hidden="true" focusable="false">
        <image href={crate::paths::asset_path(&path(persona))} width="2048" height="1024" preserveAspectRatio="none"/>
    </svg>}
}
