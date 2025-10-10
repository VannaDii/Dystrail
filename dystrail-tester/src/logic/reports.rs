use anyhow::Result;
use colored::Colorize;
use serde_json;
use std::io::Write;
use std::time::Duration;

use super::{PlayabilityRecord, ScenarioResult};

pub fn generate_console_report(
    writer: &mut dyn Write,
    results: &[ScenarioResult],
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
    #[allow(clippy::cast_precision_loss)]
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
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

    Ok(())
}

pub fn generate_json_report(writer: &mut dyn Write, results: &[ScenarioResult]) -> Result<()> {
    let json_output = serde_json::to_string_pretty(results)?;
    writeln!(writer, "{json_output}")?;
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
    #[allow(clippy::cast_precision_loss)]
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
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
        "scenario,mode,strategy,seed_code,seed_value,days_survived,ending_type,encounters_faced,vehicle_breakdowns,final_hp,final_supplies,final_sanity,final_pants,final_budget_cents"
    )?;

    for record in records {
        let metrics = &record.metrics;
        let mode = format!("{:?}", record.mode);
        let strategy = record.strategy.to_string();

        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            quote(&record.scenario_name),
            quote(&mode),
            quote(&strategy),
            quote(&record.seed_code),
            record.seed_value,
            metrics.days_survived,
            quote(&metrics.ending_type),
            metrics.encounters_faced,
            metrics.vehicle_breakdowns,
            metrics.final_hp,
            metrics.final_supplies,
            metrics.final_sanity,
            metrics.final_pants,
            metrics.final_budget_cents,
        )?;
    }

    Ok(())
}

fn quote(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}
