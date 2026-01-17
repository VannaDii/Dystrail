use crate::game::{
    GameState, OtDeluxe90sPolicy, OtDeluxeCrossingOptions, OtDeluxeRiver, OtDeluxeRiverBed,
    numbers::round_f64_to_i32, otdeluxe_crossing_options,
};
use crate::i18n;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtDeluxeCrossingViewModel {
    pub title: String,
    pub prompt: String,
    pub stats: String,
    pub ford_label: String,
    pub ford_desc: String,
    pub caulk_label: String,
    pub caulk_desc: String,
    pub ferry_label: String,
    pub ferry_desc: String,
    pub guide_label: String,
    pub guide_desc: String,
    pub back_label: String,
    pub options: OtDeluxeCrossingOptions,
}

pub fn build_otdeluxe_crossing_viewmodel(
    gs: &GameState,
) -> Result<OtDeluxeCrossingViewModel, String> {
    let river_kind = gs
        .ot_deluxe
        .crossing
        .river_kind
        .ok_or_else(|| String::from("Missing river kind"))?;
    let river_state = gs
        .ot_deluxe
        .crossing
        .river
        .as_ref()
        .ok_or_else(|| String::from("Missing river state"))?;

    let title = i18n::t("ot_cross.title");
    let river_name = i18n::t(river_name_key(river_kind));
    let mut prompt_args = BTreeMap::new();
    prompt_args.insert("river", river_name.as_str());
    let prompt = i18n::tr("ot_cross.prompt", Some(&prompt_args));

    let width = round_one_decimal(river_state.width_ft);
    let depth = round_one_decimal(river_state.depth_ft);
    let swiftness_pct = round_f64_to_i32(f64::from(river_state.swiftness) * 100.0).clamp(0, 100);
    let swiftness_pct = u8::try_from(swiftness_pct).unwrap_or_default();
    let width_str = i18n::fmt_number(f64::from(width));
    let depth_str = i18n::fmt_number(f64::from(depth));
    let swiftness_str = i18n::fmt_pct(swiftness_pct);
    let bed_str = i18n::t(bed_key(river_state.bed));
    let mut stats_args = BTreeMap::new();
    stats_args.insert("width", width_str.as_str());
    stats_args.insert("depth", depth_str.as_str());
    stats_args.insert("swiftness", swiftness_str.as_str());
    stats_args.insert("bed", bed_str.as_str());
    let stats = i18n::tr("ot_cross.stats", Some(&stats_args));

    let policy = OtDeluxe90sPolicy::default();
    let options = otdeluxe_crossing_options(
        &policy.crossings,
        river_kind,
        river_state,
        &gs.ot_deluxe.inventory,
    );

    let ford_label = i18n::t("ot_cross.options.ford");
    let ford_desc = i18n::t("ot_cross.desc.ford");
    let caulk_label = i18n::t("ot_cross.options.caulk_float");
    let caulk_desc = i18n::t("ot_cross.desc.caulk_float");
    let ferry_cost = i18n::fmt_currency(i64::from(policy.crossings.ferry_cost_cents));
    let mut ferry_args = BTreeMap::new();
    ferry_args.insert("cost", ferry_cost.as_str());
    let ferry_label = i18n::tr("ot_cross.options.ferry", Some(&ferry_args));
    let ferry_desc = i18n::t("ot_cross.desc.ferry");
    let guide_sets = policy.crossings.guide_cost_clothes_sets.to_string();
    let mut guide_args = BTreeMap::new();
    guide_args.insert("sets", guide_sets.as_str());
    let guide_label = i18n::tr("ot_cross.options.guide", Some(&guide_args));
    let guide_desc = i18n::t("ot_cross.desc.guide");
    let back_label = i18n::t("ot_cross.options.back");

    Ok(OtDeluxeCrossingViewModel {
        title,
        prompt,
        stats,
        ford_label,
        ford_desc,
        caulk_label,
        caulk_desc,
        ferry_label,
        ferry_desc,
        guide_label,
        guide_desc,
        back_label,
        options,
    })
}

const fn river_name_key(river: OtDeluxeRiver) -> &'static str {
    match river {
        OtDeluxeRiver::Kansas => "ot_cross.river.kansas",
        OtDeluxeRiver::BigBlue => "ot_cross.river.big_blue",
        OtDeluxeRiver::Green => "ot_cross.river.green",
        OtDeluxeRiver::Snake => "ot_cross.river.snake",
    }
}

const fn bed_key(bed: OtDeluxeRiverBed) -> &'static str {
    match bed {
        OtDeluxeRiverBed::Rocky => "ot_cross.bed.rocky",
        OtDeluxeRiverBed::Muddy => "ot_cross.bed.muddy",
        OtDeluxeRiverBed::Unknown => "ot_cross.bed.unknown",
    }
}

fn round_one_decimal(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}
