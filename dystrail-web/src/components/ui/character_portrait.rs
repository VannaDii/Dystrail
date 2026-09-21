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
    PortraitAsset {
        path: super::cast_art::path(persona),
        cell: Some(match expression {
            Expression::Standard => 0,
            Expression::Happy => 1,
            Expression::Defeated => 2,
        }),
    }
}

pub fn art(persona: &str, expression: Expression) -> Html {
    let pose = match expression {
        Expression::Standard => super::cast_art::Pose::Standard,
        Expression::Happy => super::cast_art::Pose::Happy,
        Expression::Defeated => super::cast_art::Pose::Defeated,
    };
    html! {<span class="character-art" data-expression={expression.key()} aria-hidden="true">
        {super::cast_art::art(persona, pose)}
    </span>}
}

pub fn framed(persona: &str, name: &str, expression: Expression) -> Html {
    html! {<figure class="character-portrait" data-member={persona.to_owned()}>
        {art(persona,expression)}<figcaption>{name}</figcaption>
    </figure>}
}
