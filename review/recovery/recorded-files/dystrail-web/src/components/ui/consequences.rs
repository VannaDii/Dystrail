//! Qualitative forecasts for choices. Completed receipts keep their exact values.
use crate::i18n;

/// An already full stat must not promise another gain. Exact prices and item offers
/// are supplied separately by the action that owns them.
pub fn stat(name: &str, change: i32, current: i32, cap: i32) -> Option<String> {
    if change == 0 {
        return None;
    }
    let direction = if change > 0 && current >= cap {
        "full"
    } else if change < 0 && current <= 0 {
        "empty"
    } else if change > 0 {
        "gain"
    } else {
        "cost"
    };
    Some(i18n::tr(
        &format!("consequences.{direction}"),
        Some(&std::collections::BTreeMap::from([("stat", i18n::t(name).as_str())])),
    ))
}

pub fn duration(minutes: u16) -> String {
    i18n::tr("consequences.duration", Some(&std::collections::BTreeMap::from([
        ("hours", i18n::fmt_number(f64::from(minutes) / 60.0).as_str())
    ])))
}

pub fn join(values: impl IntoIterator<Item = Option<String>>) -> String {
    values.into_iter().flatten().collect::<Vec<_>>().join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capped_forecasts_do_not_promise_gains_or_change_simulation() {
        crate::i18n::set_lang("en");
        assert_eq!(stat("ux.health", 2, 10, 10).as_deref(), Some("Health already full"));
        assert_eq!(stat("ux.sanity", -2, 3, 10).as_deref(), Some("Costs Sanity"));
        assert_eq!(stat("ux.sanity", 0, 3, 10), None);
    }
}
