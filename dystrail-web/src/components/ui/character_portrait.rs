//! The same identity, framing and expression source in scenes and the share image.
use crate::game::{Ending, GameState, boss::HearingOutcome};
use yew::prelude::*;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Expression {
    #[default]
    Standard,
    Happy,
    Defeated,
}

impl Expression {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Happy => "happy",
            Self::Defeated => "defeated",
        }
    }

    #[must_use]
    pub const fn hearing(outcome: HearingOutcome) -> Self {
        match outcome {
            HearingOutcome::Passed | HearingOutcome::Secured => Self::Happy,
            HearingOutcome::Failed | HearingOutcome::Exhausted => Self::Defeated,
        }
    }

    #[must_use]
    pub fn ending(state: &GameState) -> Self {
        state.boss.hearing.as_ref().map_or_else(
            || {
                if state.ending == Some(Ending::BossVictory) || state.boss.outcome.victory {
                    Self::Happy
                } else {
                    Self::Defeated
                }
            },
            |report| Self::hearing(report.outcome),
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct PortraitAsset {
    pub path: String,
    pub cell: Option<u8>,
}

#[must_use]
pub fn asset(persona: &str, expression: Expression) -> PortraitAsset {
    let persona = match persona {
        "organizer" | "whistleblower" | "lobbyist" | "staffer" | "satirist" => persona,
        _ => "journalist",
    };
    PortraitAsset {
        path: format!(
            "static/img/journey/occupant-{persona}{}.png",
            if expression == Expression::Standard {
                ""
            } else {
                "-expressions-v1"
            }
        ),
        cell: match expression {
            Expression::Standard => None,
            Expression::Happy => Some(0),
            Expression::Defeated => Some(1),
        },
    }
}

pub fn art(persona: &str, expression: Expression) -> Html {
    let portrait = asset(persona, expression);
    let path = crate::paths::asset_path(&portrait.path);
    html! {<span class="character-art" data-expression={expression.key()} aria-hidden="true">
        if let Some(cell) = portrait.cell {
            <svg viewBox={format!("{} 0 512 512", u16::from(cell)*512)} focusable="false"><image href={path} width="1024" height="512"/></svg>
        } else {<img src={path} alt="" decoding="sync"/>}
    </span>}
}

pub fn framed(persona: &str, name: &str, expression: Expression) -> Html {
    html! {<figure class="character-portrait" data-member={persona.to_owned()}>
        {art(persona,expression)}<figcaption>{name}</figcaption>
    </figure>}
}
