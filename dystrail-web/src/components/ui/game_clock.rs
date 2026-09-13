use yew::prelude::*;
#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    pub day: u32,
    #[prop_or(8)]
    pub hour: u8,
    #[prop_or_default]
    pub minute: u8,
    #[prop_or_default]
    pub moving: bool,
}
#[function_component(GameClock)]
pub fn game_clock(p: &Props) -> Html {
    crate::i18n::use_language();
    let hour = p.hour;
    let day = crate::i18n::fmt_number(f64::from(p.day));
    let time = format!("{hour:02}:{:02}", p.minute);
    html! {<span class="game-clock">{crate::i18n::tr("play2.clock",Some(&std::collections::BTreeMap::from([("day",day.as_str()),("time",time.as_str())])))}</span>}
}
