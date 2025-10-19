use anyhow::Result;
use colored::Colorize;
use serde_json;
use std::convert::TryFrom;
use std::io::Write;
use std::time::Duration;

use super::{PlayabilityAggregate, PlayabilityRecord, ScenarioResult};

#[allow(clippy::too_many_lines)]
pub fn generate_console_report(
    writer: &mut dyn Write,
    results: &[ScenarioResult],
    aggregates: &[PlayabilityAggregate],
    total_duration: Duration,
) -> Result<()> {
    writeln!(writer)?;
    writeln!(
        writer,
        "{}",
        "📊 Logic Test Results Summary".bright_cyan().bold()
    )?;
    writeln!(writer, "{}", "==============================".cyan())?;

    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;

    // Overall stats
    writeln!(writer, "Total scenarios: {total_tests}")?;
    writeln!(writer, "Passed: {}", passed_tests.to_string().green())?;
    writeln!(writer, "Failed: {}", failed_tests.to_string().red())?;
    let success_rate = if total_tests == 0 {
        0.0
    } else {
        let passed = f64::from(u32::try_from(passed_tests).unwrap_or(0));
        let total = f64::from(u32::try_from(total_tests).unwrap_or(1));
        (passed / total) * 100.0
    };
    writeln!(writer, "Success rate: {success_rate:.1}%")?;
    writeln!(writer, "Total time: {total_duration:?}")?;
    writeln!(writer)?;

    // Individual results
    for result in results {
        let status = if result.passed {
            "✅ PASS".green()
        } else {
            "❌ FAIL".red()
        };

        writeln!(writer, "{} {}", status, result.scenario_name.bold())?;
        writeln!(
            writer,
            "   Iterations: {}/{} successful",
            result.successful_iterations, result.iterations_run
        )?;
        writeln!(writer, "   Average time: {:?}", result.average_duration)?;

        if !result.failures.is_empty() {
            writeln!(writer, "   Failures:")?;
            for failure in &result.failures {
                writeln!(writer, "     • {}", failure.red())?;
            }
        }
        writeln!(writer)?;
    }

    // Performance summary
    if !results.is_empty() {
        writeln!(
            writer,
            "{}",
            "⚡ Performance Summary".bright_yellow().bold()
        )?;
        writeln!(writer, "{}", "=====================".yellow())?;

        let fastest = results.iter().min_by_key(|r| r.average_duration).unwrap();
        let slowest = results.iter().max_by_key(|r| r.average_duration).unwrap();

        writeln!(
            writer,
            "Fastest: {} ({:?})",
            fastest.scenario_name.green(),
            fastest.average_duration
        )?;
        writeln!(
            writer,
            "Slowest: {} ({:?})",
            slowest.scenario_name.yellow(),
            slowest.average_duration
        )?;
    }

    if !aggregates.is_empty() {
        writeln!(writer)?;
        writeln!(
            writer,
            "{}",
            "📈 Playability Summary".bright_magenta().bold()
        )?;
        writeln!(writer, "{}", "======================".magenta())?;

        for agg in aggregates {
            let reach_pct = agg.boss_reach_pct * 100.0;
            let win_pct = agg.boss_win_pct * 100.0;
            let pants_pct = agg.pants_failure_pct * 100.0;
            let travel_pct = agg.mean_travel_ratio * 100.0;
            let min_travel_pct = agg.min_travel_ratio * 100.0;
            let milestone_pct = agg.pct_reached_2k_by_150 * 100.0;
            let permit_pct = agg.crossing_permit_rate * 100.0;
            let bribe_succ_pct = agg.crossing_bribe_success_rate * 100.0;
            let failure_pct = agg.crossing_failure_rate * 100.0;
            let label = format!("{} ({:?}/{:?})", agg.scenario_name, agg.mode, agg.strategy);
            writeln!(
                writer,
                "• {} | n={} | days {:.1}±{:.1} | miles {:.1}±{:.1} | travel {:.1}% (min {:.1}%) | unique/20 {:.2} (min {:.2}) | ≥2k@150 {:.1}% | rotations {:.1} | boss reach {:.1}% | boss win {:.1}% | pants fails {:.1}%",
                label.bold(),
                agg.iterations,
                agg.mean_days,
                agg.std_days,
                agg.mean_miles,
                agg.std_miles,
                travel_pct,
                min_travel_pct,
                agg.mean_unique_per_20,
                agg.min_unique_per_20,
                milestone_pct,
                agg.mean_rotation_events,
                reach_pct,
                win_pct,
                pants_pct
            )?;
            writeln!(
                writer,
                "   crossings: events {:.1} | permit {:.1}% | bribes {:.2} (succ {:.1}%) | detours {:.2} | fail {:.1}%",
                agg.mean_crossing_events,
                permit_pct,
                agg.mean_crossing_bribes,
                bribe_succ_pct,
                agg.mean_crossing_detours,
                failure_pct
            )?;
            if failure_pct > 12.0 {
                writeln!(
                    writer,
                    "   {}",
                    format!("⚠ crossing failure rate {failure_pct:.1}% exceeds 12% ceiling")
                        .yellow()
                )?;
            }
            if agg.mean_crossing_bribes > 0.0 && bribe_succ_pct < 45.0 {
                writeln!(
                    writer,
                    "   {}",
                    format!("⚠ bribe success {bribe_succ_pct:.1}% below 45% target").yellow()
                )?;
            }
            if agg.mean_travel_ratio < 0.80 {
                writeln!(
                    writer,
                    "   {}",
                    format!("⚠ travel ratio {travel_pct:.1}% below 80% target").yellow()
                )?;
            }
            if agg.min_travel_ratio < 0.80 {
                writeln!(
                    writer,
                    "   {}",
                    format!("⚠ min travel ratio {min_travel_pct:.1}% below 80% requirement")
                        .yellow()
                )?;
            }
            if agg.mean_unique_per_20 < 1.5 {
                writeln!(
                    writer,
                    "   {}",
                    format!(
                        "⚠ unique encounters per 20d {:.2} below 1.5 target",
                        agg.mean_unique_per_20
                    )
                    .yellow()
                )?;
            }
            if agg.min_unique_per_20 < 1.5 {
                writeln!(
                    writer,
                    "   {}",
                    format!(
                        "⚠ min unique encounters per 20d {:.2} below 1.5 requirement",
                        agg.min_unique_per_20
                    )
                    .yellow()
                )?;
            }
            if agg.mean_miles < 2000.0 {
                writeln!(
                    writer,
                    "   {}",
                    format!("⚠ average mileage {:.0} below 2000 mi goal", agg.mean_miles).yellow()
                )?;
            }
            if milestone_pct < 25.0 {
                writeln!(
                    writer,
                    "   {}",
                    format!("⚠ only {milestone_pct:.1}% of runs reached 2,000mi by day 150")
                        .yellow()
                )?;
            }
        }
    }

    Ok(())
}

pub fn generate_json_report(writer: &mut dyn Write, results: &[ScenarioResult]) -> Result<()> {
    serde_json::to_writer_pretty(&mut *writer, results)?;
    writeln!(writer)?;
    Ok(())
}

pub fn generate_markdown_report(writer: &mut dyn Write, results: &[ScenarioResult]) -> Result<()> {
    writeln!(writer, "# Dystrail Logic Test Results\n")?;

    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;

    writeln!(writer, "## Summary\n")?;
    writeln!(writer, "- **Total scenarios**: {total_tests}")?;
    writeln!(writer, "- **Passed**: {passed_tests}")?;
    writeln!(writer, "- **Failed**: {failed_tests}")?;
    let success_rate = if total_tests == 0 {
        0.0
    } else {
        let passed = f64::from(u32::try_from(passed_tests).unwrap_or(0));
        let total = f64::from(u32::try_from(total_tests).unwrap_or(1));
        (passed / total) * 100.0
    };
    writeln!(writer, "- **Success rate**: {success_rate:.1}%\n")?;

    writeln!(writer, "## Detailed Results\n")?;

    for result in results {
        let status = if result.passed { "✅" } else { "❌" };

        writeln!(writer, "### {} {}\n", status, result.scenario_name)?;
        writeln!(
            writer,
            "- **Iterations**: {}/{} successful",
            result.successful_iterations, result.iterations_run
        )?;
        writeln!(writer, "- **Average time**: {:?}", result.average_duration)?;

        if !result.failures.is_empty() {
            writeln!(writer, "- **Failures**:")?;
            for failure in &result.failures {
                writeln!(writer, "  - {failure}")?;
            }
        }
        writeln!(writer)?;
    }

    Ok(())
}

pub fn generate_csv_report(writer: &mut dyn Write, records: &[PlayabilityRecord]) -> Result<()> {
    writeln!(
        writer,
        "scenario,mode,strategy,seed_code,seed_value,days_survived,ending_type,ending_cause,encounters_faced,vehicle_breakdowns,final_hp,final_supplies,final_sanity,final_pants,final_budget_cents,boss_reached,boss_won,miles_traveled,travel_days,partial_travel_days,non_travel_days,avg_mpd,unique_encounters,repairs_spent_cents,bribes_spent_cents,exec_order_active,exec_order_days_remaining,exec_order_cooldown,exposure_streak_heat,exposure_streak_cold,days_with_camp,days_with_repair,travel_ratio,unique_per_20_days,rotation_events,reached_2k_by_150,crossing_events,crossing_permit_uses,crossing_bribe_attempts,crossing_bribe_successes,crossing_detours_taken,crossing_failures,crossing_failure_rate,crossing_bribe_success_rate,day_reason_history,endgame_active,endgame_field_repair_used,endgame_cooldown_days,stop_cap_conversions"
    )?;

    for record in records {
        let metrics = &record.metrics;
        let strategy = record.strategy.to_string();

        let mut row = Vec::with_capacity(49);
        row.push(quote(&record.scenario_name));
        row.push(quote(mode_label(record.mode)));
        row.push(quote(&strategy));
        row.push(quote(&record.seed_code));
        row.push(record.seed_value.to_string());
        row.push(metrics.days_survived.to_string());
        row.push(quote(&metrics.ending_type));
        row.push(quote(&metrics.ending_cause));
        row.push(metrics.encounters_faced.to_string());
        row.push(metrics.vehicle_breakdowns.to_string());
        row.push(metrics.final_hp.to_string());
        row.push(metrics.final_supplies.to_string());
        row.push(metrics.final_sanity.to_string());
        row.push(metrics.final_pants.to_string());
        row.push(metrics.final_budget_cents.to_string());
        row.push(metrics.boss_reached.to_string());
        row.push(metrics.boss_won.to_string());
        row.push(format!("{:.1}", metrics.miles_traveled));
        row.push(metrics.travel_days.to_string());
        row.push(metrics.partial_travel_days.to_string());
        row.push(metrics.non_travel_days.to_string());
        row.push(format!("{:.2}", metrics.avg_miles_per_day));
        row.push(metrics.unique_encounters.to_string());
        row.push(metrics.repairs_spent_cents.to_string());
        row.push(metrics.bribes_spent_cents.to_string());
        row.push(quote(&metrics.exec_order_active));
        row.push(metrics.exec_order_days_remaining.to_string());
        row.push(metrics.exec_order_cooldown.to_string());
        row.push(metrics.exposure_streak_heat.to_string());
        row.push(metrics.exposure_streak_cold.to_string());
        row.push(metrics.days_with_camp.to_string());
        row.push(metrics.days_with_repair.to_string());
        row.push(format!("{:.3}", metrics.travel_ratio));
        row.push(format!("{:.2}", metrics.unique_per_20_days));
        row.push(metrics.rotation_events.to_string());
        row.push(metrics.reached_2000_by_day150.to_string());
        row.push(metrics.crossing_events.len().to_string());
        row.push(metrics.crossing_permit_uses.to_string());
        row.push(metrics.crossing_bribe_attempts.to_string());
        row.push(metrics.crossing_bribe_successes.to_string());
        row.push(metrics.crossing_detours_taken.to_string());
        row.push(metrics.crossing_failures.to_string());
        let crossing_events = u32::try_from(metrics.crossing_events.len()).unwrap_or(u32::MAX);
        let failure_rate = if crossing_events == 0 {
            0.0
        } else {
            f64::from(metrics.crossing_failures) / f64::from(crossing_events)
        };
        let bribe_success_rate = if metrics.crossing_bribe_attempts == 0 {
            0.0
        } else {
            f64::from(metrics.crossing_bribe_successes) / f64::from(metrics.crossing_bribe_attempts)
        };
        row.push(format!("{failure_rate:.3}"));
        row.push(format!("{bribe_success_rate:.3}"));
        row.push(quote(&metrics.day_reason_history.join("|")));
        row.push(metrics.endgame_active.to_string());
        row.push(metrics.endgame_field_repair_used.to_string());
        row.push(metrics.endgame_cooldown_days.to_string());
        row.push(metrics.stop_cap_conversions.to_string());
        writeln!(writer, "{}", row.join(","))?;
    }

    Ok(())
}

fn quote(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

const fn mode_label(mode: dystrail_game::GameMode) -> &'static str {
    match mode {
        dystrail_game::GameMode::Classic => "Classic",
        dystrail_game::GameMode::Deep => "Deep",
    }
}
