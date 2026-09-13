//! Daily presentation is derived from saved actions without changing their evidence.
use crate::game::journal::{JournalEntry, ResourceChange};
use std::collections::BTreeMap;

pub(super) struct Day<'a> {
    pub number: u32,
    pub entries: Vec<&'a JournalEntry>,
}

pub(super) struct Event<'a> {
    pub first: &'a JournalEntry,
    pub last: &'a JournalEntry,
    pub count: usize,
    pub routine: bool,
    pub settings_changed: bool,
}

pub(super) fn group(entries: &[JournalEntry]) -> Vec<Day<'_>> {
    let mut days: BTreeMap<u32, Vec<&JournalEntry>> = BTreeMap::new();
    for entry in entries {
        days.entry(entry.day).or_default().push(entry);
    }
    days.into_iter()
        .map(|(number, entries)| Day { number, entries })
        .collect()
}

impl<'a> Day<'a> {
    pub fn first(&self) -> &'a JournalEntry {
        self.entries[0]
    }

    pub fn last(&self) -> &'a JournalEntry {
        self.entries[self.entries.len() - 1]
    }

    /// Snapshots can overlap and some actions do not change a resource at all.
    /// Keep its actual first and final values instead of summing repeated deltas.
    pub fn resources(&self) -> Vec<ResourceChange> {
        let mut changes: Vec<ResourceChange> = Vec::new();
        for entry in &self.entries {
            for resource in &entry.resources {
                if let Some(change) = changes.iter_mut().find(|r| r.key == resource.key) {
                    change.after = resource.after;
                } else {
                    changes.push(resource.clone());
                }
            }
        }
        changes.retain(|change| change.before != change.after);
        changes
    }

    pub fn places(&self) -> String {
        let mut places: Vec<&str> = Vec::new();
        for entry in &self.entries {
            let place = entry.place.trim();
            if !place.is_empty() && places.last().copied() != Some(place) {
                places.push(place);
            }
        }
        places.join(" · ")
    }

    pub fn events(&self) -> Vec<Event<'a>> {
        let mut events: Vec<Event<'a>> = Vec::new();
        for entry in &self.entries {
            let routine = routine_travel(entry);
            let previous = events.last();
            let same_settings = previous.is_none_or(|previous| {
                previous.last.pace == entry.pace && previous.last.diet == entry.diet
            });
            if routine
                && same_settings
                && previous
                    .is_some_and(|previous| previous.routine && previous.last.place == entry.place)
            {
                let previous = events.last_mut().expect("a matching travel event exists");
                previous.last = entry;
                previous.count += 1;
            } else {
                events.push(Event {
                    first: entry,
                    last: entry,
                    count: 1,
                    routine,
                    settings_changed: !same_settings,
                });
            }
        }
        events
    }
}

fn routine_travel(entry: &JournalEntry) -> bool {
    entry.action_kind == "travel"
        && (entry.title.trim().is_empty()
            || ["log.traveled", "play.last_turn", "ux.day_report"]
                .iter()
                .any(|key| entry.title.trim() == crate::i18n::t(key)))
        && crate::i18n::meaningful_message(&entry.message).is_empty()
        && !entry
            .details
            .iter()
            .any(|(name, _)| crate::app::receipt::narrative_detail(name))
}
