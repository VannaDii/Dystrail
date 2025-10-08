mod browser;
mod logic;
mod common;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::time::Instant;

use browser::{TestBridge, BrowserConfig, BrowserKind, new_session};
use logic::LogicTester;
use common::{artifacts_dir, capture_artifacts, split_csv};
use common::scenario::{ScenarioCtx, get_scenario};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TestMode {
    /// Pure game logic testing (fast, no browser)
    Logic,
    /// Browser automation testing (slow, captures screenshots)
    Browser,
    /// Run both logic and browser tests
    Both,
}

#[derive(Debug, Parser)]
#[command(name = "dystrail-tester", version = "0.10.0")]
#[command(
    about = "Automated QA testing for Dystrail game - both pure logic and browser automation"
)]
struct Args {
    /// Test mode: logic (fast), browser (visual), or both
    #[arg(long, value_enum, default_value_t = TestMode::Logic)]
    mode: TestMode,

    /// Scenarios to run (comma-separated)
    #[arg(long, default_value = "smoke")]
    scenarios: String,

    /// Seeds to run (comma-separated)
    #[arg(long, default_value = "1337")]
    seeds: String,

    /// Number of iterations per scenario (logic mode only)
    #[arg(long, default_value_t = 10)]
    iterations: usize,

    /// Output report format
    #[arg(long, default_value = "console")]
    #[arg(value_parser = ["json", "markdown", "console"])]
    report: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    // Browser-specific options
    /// Browsers to run (chrome,edge,firefox,safari) - browser mode only
    #[arg(long, default_value = "chrome")]
    browsers: String,

    /// Base URL of the game (should include ?test=1 to expose the bridge)
    #[arg(long, default_value = "http://localhost:5173/?test=1")]
    base_url: String,

    /// Artifacts directory for screenshots and logs
    #[arg(long, default_value = "target/test-artifacts")]
    artifacts_dir: String,

    /// Connect to a Selenium Grid/Appium hub instead of local drivers
    #[arg(long)]
    hub: Option<String>,

    /// Run headless where supported
    #[arg(long, default_value_t = true)]
    headless: bool,
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    println!("{}", "🎮 Dystrail Automated Tester".bright_cyan().bold());
    println!("{}", "================================".cyan());

    let start_time = Instant::now();
    let mut scenarios = split_csv(&args.scenarios);

    // Expand 'all' to include all comprehensive test scenarios
    if scenarios.contains(&"all".to_string()) {
        scenarios.retain(|s| s != "all");
        scenarios.extend_from_slice(&[
            "smoke".to_string(),
            "basic-game-creation".to_string(),
            "share-code-consistency".to_string(),
            "deterministic-gameplay".to_string(),
            "encounter-choices".to_string(),
            "vehicle-system".to_string(),
            "weather-effects".to_string(),
            "resource-management".to_string(),
            "stats-boundaries".to_string(),
            "inventory-operations".to_string(),
            "game-mode-variations".to_string(),
        ]);
    }

    let seeds: Vec<i64> = split_csv(&args.seeds)
        .into_iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    let mut all_results: Vec<logic::ScenarioResult> = Vec::new();

    // Run logic tests if requested
    if matches!(args.mode, TestMode::Logic | TestMode::Both) {
        println!("{}", "🧠 Running Logic Tests".bright_yellow().bold());
        println!("{}", "-".repeat(30).yellow());

        let mut logic_tester = LogicTester::new(args.verbose);

        for scenario_name in &scenarios {
            if let Some(combined_scenario) = get_scenario(scenario_name) {
                if let Some(logic_scenario) = combined_scenario.as_logic_scenario() {
                    let results =
                        logic_tester.run_scenario(&logic_scenario, &seeds, args.iterations);
                    all_results.extend(results);
                } else {
                    eprintln!(
                        "⚠️  Scenario {} has no logic test implementation",
                        scenario_name.yellow()
                    );
                }
            } else {
                eprintln!("⚠️  Unknown scenario: {}", scenario_name.yellow());
            }
        }
    }

    // Run browser tests if requested
    if matches!(args.mode, TestMode::Browser | TestMode::Both) {
        println!("{}", "🌐 Running Browser Tests".bright_blue().bold());
        println!("{}", "-".repeat(30).blue());

        let browsers = split_csv(&args.browsers);

        for browser_name in browsers {
            let kind = match browser_name.as_str() {
                "chrome" => BrowserKind::Chrome,
                "edge" => BrowserKind::Edge,
                "firefox" => BrowserKind::Firefox,
                "safari" => BrowserKind::Safari,
                other => {
                    eprintln!("⚠️  Unknown browser: {}", other.yellow());
                    continue;
                }
            };

            let cfg = BrowserConfig {
                headless: args.headless,
                implicit_wait_secs: 3,
                remote_hub: args.hub.clone(),
            };

            let driver = match new_session(kind, &cfg).await {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Could not start {kind:?}: {e}");
                    continue;
                }
            };

            for scenario_name in &scenarios {
                if let Some(scenario) = get_scenario(scenario_name) {
                    for &seed in &seeds {
                        let bridge = TestBridge::new(&driver);
                        let ctx = ScenarioCtx {
                            base_url: args.base_url.clone(),
                            seed,
                            bridge,
                            verbose: args.verbose,
                        };

                        let label = format!("{kind:?}").to_lowercase();
                        let dir = artifacts_dir(&args.artifacts_dir, &label, scenario_name, seed);

                        let scenario_start = Instant::now();
                        match scenario.run_browser(&driver, &ctx).await {
                            Ok(()) => {
                                let duration = scenario_start.elapsed();
                                println!(
                                    "✅ [{} seed {}] {} - {:?}",
                                    label.green(),
                                    seed,
                                    scenario_name,
                                    duration
                                );
                            }
                            Err(e) => {
                                let duration = scenario_start.elapsed();
                                eprintln!(
                                    "❌ [{} seed {}] {} - {:?}: {:#}",
                                    label.red(),
                                    seed,
                                    scenario_name,
                                    duration,
                                    e
                                );
                                let _ = capture_artifacts(&driver, &dir, &e).await;
                            }
                        }
                    }
                }
            }

            let _ = driver.quit().await;
        }
    }

    let duration = start_time.elapsed();

    // Generate reports (logic results only for now)
    if !all_results.is_empty() {
        match args.report.as_str() {
            "json" => logic::reports::generate_json_report(&all_results)?,
            "markdown" => logic::reports::generate_markdown_report(&all_results),
            _ => logic::reports::generate_console_report(&all_results, duration),
        }
    }

    println!();
    println!("🏁 Total time: {duration:?}");

    // Exit with error code if any tests failed
    let failed_tests = all_results.iter().any(|r| !r.passed);
    if failed_tests {
        std::process::exit(1);
    }

    Ok(())
}
