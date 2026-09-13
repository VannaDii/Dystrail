mod day;
mod view;

#[cfg(test)]
mod tests;

use crate::{game::GameState, i18n};
use yew::prelude::*;

pub fn render(gs: &GameState, logs: &[String]) -> Html {
    let days = day::group(&gs.continuity.journal);
    html! {<section class="journal" aria-label={i18n::t("play.journal")}>
        <ol class="trail-log" role="log" tabindex="0" aria-label={i18n::t("play.journal")} aria-relevant="additions text">
            {for days.iter().rev().map(view::day)}
            if days.is_empty() {
                {for logs.iter().rev().take(30).map(|line|html!{<li class="journal-legacy">{i18n::log_message(line)}</li>})}
                if logs.is_empty() {<li class="journal-legacy">{i18n::t("play.journal_empty")}</li>}
            }
        </ol>
    </section>}
}
