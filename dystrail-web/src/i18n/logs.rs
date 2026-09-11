//! Resolve legacy engine logs without exposing keys or unfilled templates.
#[must_use]
pub fn log_message(line: &str) -> String {
    if !line.starts_with("log.") && !line.starts_with("exec.") {
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
        "log.crossing.passed" => "play_log.crossed",
        "log.crossing.failure" => "play_log.crossing_failed",
        "log.crossing.decision.bribe" => "play_log.bribe",
        "log.crossing.decision.permit" => "play_log.permit",
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
    fn keys_and_placeholders_do_not_reach_players() {
        crate::i18n::set_lang("en");
        for k in [
            "log.pants-emergency",
            "log.breakdown-repaired",
            "log.run_begins",
            "log.missing",
        ] {
            let s = super::log_message(k);
            assert!(!s.starts_with("log."));
            assert!(!s.contains('{'));
        }
    }
}
