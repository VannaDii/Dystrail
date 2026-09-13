//! Resolve legacy engine logs without exposing keys or unfilled templates.
/// Remove transition announcements already conveyed by the scene and route meter.
#[must_use]
pub fn meaningful_message(message: &str) -> String {
    let mut message = message.to_owned();
    for key in ["play_log.encounter", "play_log.partial", "log.encounter"] {
        let announcement = super::t(key);
        if announcement != key && !announcement.is_empty() {
            message = message.replace(&announcement, "");
        }
    }
    if message.trim() == super::t("log.traveled") {
        String::new()
    } else {
        message.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

#[must_use]
pub fn log_message(line: &str) -> String {
    if !line.starts_with("log.") && !line.starts_with("exec.") && !line.starts_with("crossing.") {
        return line.to_owned();
    }
    let normalized = line.replace('-', "_");
    for key in [line, normalized.as_str()] {
        let text = super::t(key);
        if text != key && !text.contains('{') {
            return text;
        }
    }
    let key = match line {
        "log.breakdown-repaired" | "log.vehicle.repair.spare" => "play_log.spare",
        "log.breakdown-jury-rigged" => "play_log.jury",
        "log.travel-blocked" => "play_log.blocked",
        "log.vehicle.repair.forced"
        | "log.vehicle.field-repair-guard"
        | "log.vehicle.emergency-limp" => "play_log.service",
        "log.vehicle.da-field-repair" | "log.endgame.field-repair" => "play_log.field",
        "log.travel.partial"
        | "log.travel.rest-credit"
        | "log.travel.delay-credit"
        | "log.travel.bonus" => "play_log.partial",
        "log.crossing.detour" => "play_log.detour",
        "log.crossing.denied" => "play_log.checkpoint_denied",
        "log.crossing.passed" => "play_log.crossed",
        "log.crossing.failure" => "play_log.crossing_failed",
        "log.crossing.decision.bribe" => "play_log.bribe",
        "log.crossing.decision.permit" => "play_log.permit",
        "crossing.result.bribe.success" => "play_log.bribe_success",
        "crossing.result.bribe.fail" => "play_log.bribe_failed",
        "log.encounter.rotation" => "play_log.encounter",
        "log.endgame.activate" | "log.endgame.guard" => "play_log.capitol",
        "log.boss.victory" => "play_log.victory",
        "log.boss.failure" => "play_log.defeat",
        _ => "play.unknown_log",
    };
    super::t(key)
}
#[cfg(test)]
mod tests {
    #[test]
    fn receipts_keep_consequences_without_repeating_scene_transitions() {
        crate::i18n::set_lang("en");
        let message = "The next encounter is ready. You made progress along the route. Encounter!";
        assert!(super::meaningful_message(message).is_empty());
        let consequence = "Illness strikes the crew.";
        assert_eq!(
            super::meaningful_message(&format!("{message} {consequence}")),
            consequence
        );
    }

    #[test]
    fn keys_and_placeholders_do_not_reach_players() {
        crate::i18n::set_lang("en");
        for k in [
            "log.breakdown-repaired",
            "log.run_begins",
            "log.missing",
            "crossing.result.bribe.success",
            "crossing.result.bribe.fail",
        ] {
            let s = super::log_message(k);
            assert!(!s.starts_with("log."));
            assert!(!s.starts_with("crossing."));
            assert!(!s.starts_with("play_log."));
            assert!(!s.contains('{'));
        }
    }
}
