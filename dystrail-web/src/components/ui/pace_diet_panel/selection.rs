use crate::game::{DietId, PaceId, PacingConfig};
use crate::i18n;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionOutcome {
    Pace(PaceId, String),
    Diet(DietId, String),
}

fn pace_message(pacing_config: &PacingConfig, pace_id: PaceId) -> SelectionOutcome {
    let pace_cfg = pacing_config.get_pace_safe(pace_id.as_str());
    let sanity_str = format!("{:+}", pace_cfg.sanity);
    let pants_str = format!("{:+}", pace_cfg.pants);
    let chance_str = format!("{:+.0}%", pace_cfg.encounter_chance_delta * 100.0);
    let mut args = BTreeMap::new();
    args.insert("pace", pace_cfg.name.as_str());
    args.insert("sanity", sanity_str.as_str());
    args.insert("pants", pants_str.as_str());
    args.insert("chance", chance_str.as_str());
    let msg = i18n::tr("pacediet.announce.pace_set", Some(&args));
    SelectionOutcome::Pace(pace_id, msg)
}

fn diet_message(pacing_config: &PacingConfig, diet_id: DietId) -> SelectionOutcome {
    let diet_cfg = pacing_config.get_diet_safe(diet_id.as_str());
    let sanity_str = format!("{:+}", diet_cfg.sanity);
    let pants_str = format!("{:+}", diet_cfg.pants);
    let receipt_str = format!("{:+}%", diet_cfg.receipt_find_pct_delta);
    let mut args = BTreeMap::new();
    args.insert("diet", diet_cfg.name.as_str());
    args.insert("sanity", sanity_str.as_str());
    args.insert("pants", pants_str.as_str());
    args.insert("receipt", receipt_str.as_str());
    let msg = i18n::tr("pacediet.announce.diet_set", Some(&args));
    SelectionOutcome::Diet(diet_id, msg)
}

#[must_use]
pub fn selection_outcome(pacing_config: &PacingConfig, idx: u8) -> Option<SelectionOutcome> {
    match idx {
        1 => Some(pace_message(pacing_config, PaceId::Steady)),
        2 => Some(pace_message(pacing_config, PaceId::Heated)),
        3 => Some(pace_message(pacing_config, PaceId::Blitz)),
        4 => Some(diet_message(pacing_config, DietId::Quiet)),
        5 => Some(diet_message(pacing_config, DietId::Mixed)),
        6 => Some(diet_message(pacing_config, DietId::Doom)),
        _ => None,
    }
}
