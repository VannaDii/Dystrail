use anyhow::Result;
use colored::Colorize;
use serde_json;
use std::convert::TryFrom;
use std::io::Write;
use std::time::Duration;

use super::{PlayabilityAggregate, PlayabilityRecord, ScenarioResult};

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
            let label = format!("{} ({:?}/{:?})", agg.scenario_name, agg.mode, agg.strategy);
            writeln!(
                writer,
                "• {} | n={} | days {:.1}±{:.1} | miles {:.1}±{:.1} | boss reach {:.1}% | boss win {:.1}% | pants fails {:.1}%",
                label.bold(),
                agg.iterations,
                agg.mean_days,
                agg.std_days,
                agg.mean_miles,
                agg.std_miles,
                reach_pct,
                win_pct,
                pants_pct
            )?;
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
        "scenario,mode,strategy,seed_code,seed_value,days_survived,ending_type,ending_cause,encounters_faced,vehicle_breakdowns,final_hp,final_supplies,final_sanity,final_pants,final_budget_cents,boss_reached,boss_won,miles_traveled,travel_days,non_travel_days,avg_mpd,unique_encounters,repairs_spent_cents,bribes_spent_cents,exec_order_active,exec_order_days_remaining,exec_order_cooldown,exposure_streak_heat,exposure_streak_cold,days_with_camp,days_with_repair"
    )?;

    for record in records {
        let metrics = &record.metrics;
        let strategy = record.strategy.to_string();

        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.1},{},{},{:.2},{},{},{},{},{},{},{},{},{},{},{}",
            quote(&record.scenario_name),
            quote(mode_label(record.mode)),
            quote(&strategy),
            quote(&record.seed_code),
            record.seed_value,
            metrics.days_survived,
            quote(&metrics.ending_type),
            quote(&metrics.ending_cause),
            metrics.encounters_faced,
            metrics.vehicle_breakdowns,
            metrics.final_hp,
            metrics.final_supplies,
            metrics.final_sanity,
            metrics.final_pants,
            metrics.final_budget_cents,
            metrics.boss_reached,
            metrics.boss_won,
            metrics.miles_traveled,
            metrics.travel_days,
            metrics.non_travel_days,
            metrics.avg_miles_per_day,
            metrics.unique_encounters,
            metrics.repairs_spent_cents,
            metrics.bribes_spent_cents,
            quote(&metrics.exec_order_active),
            metrics.exec_order_days_remaining,
            metrics.exec_order_cooldown,
            metrics.exposure_streak_heat,
            metrics.exposure_streak_cold,
            metrics.days_with_camp,
            metrics.days_with_repair,
        )?;
    }

    Ok(())
}

fn quote(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn mode_label(mode: dystrail_game::GameMode) -> &'static str {
    match mode {
        dystrail_game::GameMode::Classic => "Classic",
        dystrail_game::GameMode::Deep => "Deep",
    }
}
