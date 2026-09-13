use super::day::{Day, Event};
use crate::components::ui::{journey_icon, stat_card};
use crate::game::journal::JournalEntry;
use crate::i18n;
use std::collections::BTreeMap;
use yew::prelude::*;

pub(super) fn day(day: &Day<'_>) -> Html {
    let number = i18n::fmt_number(f64::from(day.number));
    let heading = i18n::tr(
        "route.day",
        Some(&BTreeMap::from([("day", number.as_str())])),
    );
    let first = day.first();
    let last = day.last();
    let resources = day.resources();
    let count = i18n::fmt_number(f64::from(
        u32::try_from(day.entries.len()).unwrap_or(u32::MAX),
    ));
    html! {<li key={day.number} class="journal-entry journal-day" data-day={day.number.to_string()}>
        <article class="journal-story">
            <div class="journal-narrative">
                <header>
                    <h3>{heading}</h3>
                    <p class="journal-place">{day.places()}</p>
                    <div class="journal-context">
                        <span class="journal-stamp">{journey_icon::render("clock")}{time(last.minute)}</span>
                        {settings(first)}
                    </div>
                </header>
                <ol class="journal-events">
                    {for day.events().iter().map(event)}
                </ol>
            </div>
            {stat_card::render_changes(&first.before, &last.after, &resources)}
        </article>
        <details class="journal-raw">
            <summary>{i18n::t("ux.day_events")}<span>{count}</span></summary>
            <ol class="journal-raw-list">
                {for day.entries.iter().map(|entry| raw_entry(entry))}
            </ol>
        </details>
    </li>}
}

fn event(event: &Event<'_>) -> Html {
    let entry = event.first;
    html! {<li class="journal-event" data-action={entry.action_kind.clone()} data-action-count={event.count.to_string()}>
        {journey_icon::render(action_icon(entry))}
        <div class="journal-event-copy">
            <p class="journal-event-heading">{heading(entry)}</p>
            <div class="journal-event-context">
                <span class="journal-event-time">{time(event.last.minute)}</span>
                if event.settings_changed { {settings(entry)} }
            </div>
            {details(entry)}
        </div>
    </li>}
}

fn raw_entry(entry: &JournalEntry) -> Html {
    let title = if entry.title.trim().is_empty() {
        heading(entry)
    } else {
        entry.title.clone()
    };
    html! {<li class="journal-raw-entry" data-action={entry.action_kind.clone()}>
        <div class="journal-raw-story">
            <h4>{journey_icon::render(action_icon(entry))}{title.clone()}</h4>
            if !entry.message.trim().is_empty() && entry.message.trim() != title {
                <p class="journal-raw-message">{&entry.message}</p>
            }
            <p class="journal-place">{&entry.place}</p>
            <div class="journal-event-context">
                <span>{journey_icon::render("clock")}{time(entry.minute)}</span>
                {settings(entry)}
            </div>
            {details(entry)}
        </div>
        {stat_card::render_changes(&entry.before, &entry.after, &entry.resources)}
    </li>}
}

fn details(entry: &JournalEntry) -> Html {
    html! {<dl class="journal-details">
        {for entry.details.iter().filter(|(name,_)|crate::app::receipt::narrative_detail(name)).map(|(name,value)|html!{<div><dt>{name}</dt><dd>{value}</dd></div>})}
    </dl>}
}

fn heading(entry: &JournalEntry) -> String {
    let message = i18n::meaningful_message(&entry.message);
    if !message.is_empty() {
        return message;
    }
    let title = entry.title.trim();
    if !title.is_empty() && title != i18n::t("play.last_turn") {
        return title.to_owned();
    }
    i18n::t(match entry.action_kind.as_str() {
        "travel" => "log.traveled",
        "repair" => "play.repairing",
        "camp" => "ux.camp",
        "town" => "journey.arrival",
        "care" => "journey.crew_stop",
        _ => "ux.day_report",
    })
}

fn action_icon(entry: &JournalEntry) -> &str {
    if entry.action_kind.is_empty() {
        "encounter"
    } else {
        &entry.action_kind
    }
}

fn settings(entry: &JournalEntry) -> Html {
    html! {<>
        if let Some(pace) = entry.pace { {setting("play.pace", pace.as_str())} }
        if let Some(diet) = entry.diet { {setting("play.diet", diet.as_str())} }
    </>}
}

fn setting(kind: &str, value: &str) -> Html {
    let name = i18n::t(&format!("play.{value}"));
    html! {<span aria-label={format!("{}: {name}",i18n::t(kind))}>
        {journey_icon::render(value)}{name}
    </span>}
}

fn time(minute: u16) -> String {
    format!("{:02}:{:02}", minute / 60, minute % 60)
}
