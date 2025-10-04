use anyhow::Result;
use colored::Colorize;
use serde_json;
use std::time::Duration;

use super::ScenarioResult;

pub fn generate_console_report(results: &[ScenarioResult], total_duration: Duration) {
    println!();
    println!("{}", "📊 Logic Test Results Summary".bright_cyan().bold());
    println!("{}", "==============================".cyan());

    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;

    // Overall stats
    println!("Total scenarios: {total_tests}");
    println!("Passed: {}", passed_tests.to_string().green());
    println!("Failed: {}", failed_tests.to_string().red());
    #[allow(clippy::cast_precision_loss)]
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    println!("Success rate: {success_rate:.1}%");
    println!("Total time: {total_duration:?}");
    println!();

    // Individual results
    for result in results {
        let status = if result.passed {
            "✅ PASS".green()
        } else {
            "❌ FAIL".red()
        };

        println!("{} {}", status, result.scenario_name.bold());
        println!(
            "   Iterations: {}/{} successful",
            result.successful_iterations, result.iterations_run
        );
        println!("   Average time: {:?}", result.average_duration);

        if !result.failures.is_empty() {
            println!("   Failures:");
            for failure in &result.failures {
                println!("     • {}", failure.red());
            }
        }
        println!();
    }

    // Performance summary
    if !results.is_empty() {
        println!("{}", "⚡ Performance Summary".bright_yellow().bold());
        println!("{}", "=====================".yellow());

        let fastest = results.iter().min_by_key(|r| r.average_duration).unwrap();
        let slowest = results.iter().max_by_key(|r| r.average_duration).unwrap();

        println!(
            "Fastest: {} ({:?})",
            fastest.scenario_name.green(),
            fastest.average_duration
        );
        println!(
            "Slowest: {} ({:?})",
            slowest.scenario_name.yellow(),
            slowest.average_duration
        );
    }
}

pub fn generate_json_report(results: &[ScenarioResult]) -> Result<()> {
    let json_output = serde_json::to_string_pretty(results)?;
    println!("{json_output}");
    Ok(())
}

pub fn generate_markdown_report(results: &[ScenarioResult]) {
    println!("# Dystrail Logic Test Results\n");

    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let failed_tests = total_tests - passed_tests;

    println!("## Summary\n");
    println!("- **Total scenarios**: {total_tests}");
    println!("- **Passed**: {passed_tests}");
    println!("- **Failed**: {failed_tests}");
    #[allow(clippy::cast_precision_loss)]
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    println!("- **Success rate**: {success_rate:.1}%\n");

    println!("## Detailed Results\n");

    for result in results {
        let status = if result.passed { "✅" } else { "❌" };

        println!("### {} {}\n", status, result.scenario_name);
        println!(
            "- **Iterations**: {}/{} successful",
            result.successful_iterations, result.iterations_run
        );
        println!("- **Average time**: {:?}", result.average_duration);

        if !result.failures.is_empty() {
            println!("- **Failures**:");
            for failure in &result.failures {
                println!("  - {failure}");
            }
        }
        println!();
    }
}
