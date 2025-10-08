use colored::Colorize;use colored::Colorize;

use serde_json;use std::time::Duration;

use std::time::Duration;

use crate::common::scenarios::ScenarioResult;

use crate::common::scenarios::ScenarioResult;    println!();

    println!("{}", "📊 Test Results Summary".bright_cyan().bold());

pub fn generate_console_report(results: &[ScenarioResult], total_duration: Duration) {    println!("{}", "======================".cyan());

    println!();

    println!("{}", "📊 Test Results Summary".bright_cyan().bold());    let total_tests = results.len();

    println!("{}", "======================".cyan());    let passed_tests = results.iter().filter(|r| r.passed).count();

    let failed_tests = total_tests - passed_tests;

    let total_tests = results.len();

    let passed_tests = results.iter().filter(|r| r.passed).count();    // Overall stats

    let failed_tests = total_tests - passed_tests;    println!("Total scenarios: {total_tests}");

    println!("Passed: {}", passed_tests.to_string().green());

    // Overall stats    println!("Failed: {}", failed_tests.to_string().red());

    println!("Total scenarios: {total_tests}");

    println!("Passed: {}", passed_tests.to_string().green());    #[allow(clippy::cast_precision_loss)]

    println!("Failed: {}", failed_tests.to_string().red());    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;

        println!("Success rate: {success_rate:.1}%");

    #[allow(clippy::cast_precision_loss)]    println!("Total time: {total_duration:?}");

    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;    println!();

    println!("Success rate: {success_rate:.1}%");

    println!("Total time: {total_duration:?}");    // Individual results

    println!();    for result in results {

        let status = if result.passed {

    // Individual results            "✅ PASS".green()

    for result in results {        } else {

        let status = if result.passed {            "❌ FAIL".red()

            "✅ PASS".green()        };

        } else {

            "❌ FAIL".red()        println!("{} {}", status, result.scenario_name.bold());

        };        println!("   Iterations: {}/{} successful",

            result.successful_iterations,

        println!("{status} {}", result.scenario_name.bold());            result.iterations_run

        println!("   Iterations: {}/{} successful",        );

            result.successful_iterations,        println!("   Average time: {:?}", result.average_duration);

            result.iterations_run

        );        if !result.failures.is_empty() {

        println!("   Average time: {:?}", result.average_duration);            println!("   Failures:");

            for failure in &result.failures {

        if !result.failures.is_empty() {                println!("     • {}", failure.red());

            println!("   Failures:");            }

            for failure in &result.failures {        }

                println!("     • {}", failure.red());        println!();

            }    }

        }

        println!();    // Performance summary

    }    if !results.is_empty() {

        println!("{}", "⚡ Performance Summary".bright_yellow().bold());

    // Performance summary        println!("{}", "=====================".yellow());

    if !results.is_empty() {

        println!("{}", "⚡ Performance Summary".bright_yellow().bold());        let fastest = results.iter()

        println!("{}", "=====================".yellow());            .min_by_key(|r| r.average_duration)

            .unwrap();

        let fastest = results.iter()        let slowest = results.iter()

            .min_by_key(|r| r.average_duration)            .max_by_key(|r| r.average_duration)

            .expect("results is not empty");            .unwrap();

        let slowest = results.iter()

            .max_by_key(|r| r.average_duration)        println!("Fastest: {} ({:?})", fastest.scenario_name.green(), fastest.average_duration);

            .expect("results is not empty");        println!("Slowest: {} ({:?})", slowest.scenario_name.yellow(), slowest.average_duration);

    }

        println!("Fastest: {} ({:?})", fastest.scenario_name.green(), fastest.average_duration);

        println!("Slowest: {} ({:?})", slowest.scenario_name.yellow(), slowest.average_duration);    Ok(())

    }}

}

pub fn generate_json_report(results: &[ScenarioResult]) -> Result<()> {

pub fn generate_json_report(results: &[ScenarioResult]) -> Result<(), Box<dyn std::error::Error>> {    let json_output = serde_json::to_string_pretty(results)?;

    let json_output = serde_json::to_string_pretty(results)?;    println!("{}", json_output);

    println!("{json_output}");    Ok(())

    Ok(())}

}

pub fn generate_markdown_report(results: &[ScenarioResult]) -> Result<()> {

pub fn generate_markdown_report(results: &[ScenarioResult]) {    println!("# Dystrail Test Results\n");

    println!("# Dystrail Test Results\n");

    let total_tests = results.len();

    let total_tests = results.len();    let passed_tests = results.iter().filter(|r| r.passed).count();

    let passed_tests = results.iter().filter(|r| r.passed).count();    let failed_tests = total_tests - passed_tests;

    let failed_tests = total_tests - passed_tests;

    println!("## Summary\n");

    println!("## Summary\n");    println!("- **Total scenarios**: {}", total_tests);

    println!("- **Total scenarios**: {total_tests}");    println!("- **Passed**: {}", passed_tests);

    println!("- **Passed**: {passed_tests}");    println!("- **Failed**: {}", failed_tests);

    println!("- **Failed**: {failed_tests}");    println!("- **Success rate**: {:.1}%\n", (passed_tests as f64 / total_tests as f64) * 100.0);



    #[allow(clippy::cast_precision_loss)]    println!("## Detailed Results\n");

    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;

    println!("- **Success rate**: {success_rate:.1}%\n");    for result in results {

        let status = if result.passed { "✅" } else { "❌" };

    println!("## Detailed Results\n");

        println!("### {} {}\n", status, result.scenario_name);

    for result in results {        println!("- **Iterations**: {}/{} successful", result.successful_iterations, result.iterations_run);

        let status = if result.passed { "✅" } else { "❌" };        println!("- **Average time**: {:?}", result.average_duration);



        println!("### {status} {}\n", result.scenario_name);        if !result.failures.is_empty() {

        println!("- **Iterations**: {}/{} successful", result.successful_iterations, result.iterations_run);            println!("- **Failures**:");

        println!("- **Average time**: {:?}", result.average_duration);            for failure in &result.failures {

                println!("  - {}", failure);

        if !result.failures.is_empty() {            }

            println!("- **Failures**:");        }

            for failure in &result.failures {        println!();

                println!("  - {failure}");    }

            }

        }    Ok(())

        println!();}
    }
}