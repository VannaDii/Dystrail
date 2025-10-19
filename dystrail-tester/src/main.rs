mod browser;
mod common;
mod logic;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::path::PathBuf;
use std::time::Instant;

use browser::{BrowserConfig, BrowserKind, TestBridge, new_session};
use common::scenario::{ScenarioCtx, get_scenario, list_scenarios};
use common::{artifacts_dir, capture_artifacts, split_csv};
use logic::{
    LogicTester, PlayabilityAggregate, PlayabilityRecord, aggregate_playability,
    resolve_seed_inputs, run_playability_analysis, validate_playability_targets,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TestMode {
    /// Pure game logic testing (fast, no browser)
    Logic,
    /// Browser automation testing (slow, captures screenshots)
    Browser,
    /// Run both logic and browser tests
    Both,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HeadlessMode {
    /// Run browsers in headless mode
    Headless,
    /// Run browsers with visible windows
    Windowed,
}

impl HeadlessMode {
    const fn is_headless(self) -> bool {
        matches!(self, Self::Headless)
    }
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

    /// List all available scenarios and exit
    #[arg(long)]
    list_scenarios: bool,

    /// Seeds to run (comma-separated)
    #[arg(long, default_value = "1337")]
    seeds: String,

    /// Number of iterations per scenario (logic mode only)
    #[arg(long, default_value_t = 10)]
    iterations: usize,

    /// Run extended acceptance sweeps (forces ≥100 iterations for playability analysis)
    #[arg(long)]
    acceptance: bool,

    /// Output report format
    #[arg(long, default_value = "console")]
    #[arg(value_parser = ["json", "markdown", "console", "csv"])]
    report: String,

    /// Verbose output
    #[arg(short, long, alias = "versbose")]
    verbose: bool,

    /// Optional path to write the report output instead of stdout
    #[arg(long)]
    output: Option<PathBuf>,

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
    #[arg(long, value_enum, default_value_t = HeadlessMode::Headless)]
    headless: HeadlessMode,
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if args.list_scenarios {
        let mut output_target = OutputTarget::new(args.output.clone())?;
        writeln!(output_target.writer(), "Available scenarios:")?;
        for (key, description) in list_scenarios() {
            writeln!(output_target.writer(), "  {key:25} - {description}")?;
        }
        output_target.flush_inner()?;
        return Ok(());
    }

    println!("{}", "🎮 Dystrail Automated Tester".bright_cyan().bold());
    println!("{}", "================================".cyan());

    let playability_iterations = if args.acceptance {
        if args.iterations < 100 {
            println!(
                "🔁 Acceptance mode enabled: increasing playability iterations from {} to 100",
                args.iterations
            );
        } else {
            println!(
                "🔁 Acceptance mode enabled: using {} playability iterations",
                args.iterations
            );
        }
        args.iterations.max(100)
    } else {
        args.iterations
    };

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

    let seed_tokens = split_csv(&args.seeds);
    let seed_infos = resolve_seed_inputs(&seed_tokens)?;
    let logic_seeds: Vec<u64> = seed_infos.iter().map(|s| s.seed).collect();

    let mut all_results: Vec<logic::ScenarioResult> = Vec::new();

    // Run logic tests if requested
    if matches!(args.mode, TestMode::Logic | TestMode::Both) {
        println!("{}", "🧠 Running Logic Tests".bright_yellow().bold());
        println!("{}", "-".repeat(30).yellow());

        let logic_tester = LogicTester::new(args.verbose);

        for scenario_name in &scenarios {
            if let Some(combined_scenario) = get_scenario(scenario_name) {
                if let Some(logic_scenario) = combined_scenario.as_logic_scenario() {
                    let results =
                        logic_tester.run_scenario(&logic_scenario, &logic_seeds, args.iterations);
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
                headless: args.headless.is_headless(),
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
                    for seed_info in &seed_infos {
                        let bridge = TestBridge::new(&driver);
                        let ctx = ScenarioCtx {
                            base_url: args.base_url.clone(),
                            seed: seed_info.seed,
                            bridge,
                            verbose: args.verbose,
                        };

                        let label = format!("{kind:?}").to_lowercase();
                        let dir = artifacts_dir(
                            &args.artifacts_dir,
                            &label,
                            scenario_name,
                            seed_info.seed,
                        );

                        let scenario_start = Instant::now();
                        match scenario.run_browser(&driver, &ctx).await {
                            Ok(()) => {
                                let duration = scenario_start.elapsed();
                                println!(
                                    "✅ [{} seed {}] {} - {:?}",
                                    label.green(),
                                    seed_info.seed,
                                    scenario_name,
                                    duration
                                );
                            }
                            Err(e) => {
                                let duration = scenario_start.elapsed();
                                eprintln!(
                                    "❌ [{} seed {}] {} - {:?}: {:#}",
                                    label.red(),
                                    seed_info.seed,
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

    let mut playability_records: Option<Vec<PlayabilityRecord>> = None;
    let mut playability_aggregates: Option<Vec<PlayabilityAggregate>> = None;
    let require_playability = matches!(args.report.as_str(), "console" | "csv")
        || matches!(args.mode, TestMode::Logic | TestMode::Both);

    if require_playability {
        let playability =
            run_playability_analysis(&seed_infos, playability_iterations, args.verbose)?;
        playability_aggregates = Some(aggregate_playability(&playability));
        playability_records = Some(playability);
    }

    let mut output_target = OutputTarget::new(args.output.clone())?;

    match args.report.as_str() {
        "json" => {
            if all_results.is_empty() {
                writeln!(&mut output_target, "[]")?;
            } else {
                logic::reports::generate_json_report(&mut output_target, &all_results)?;
            }
        }
        "markdown" => {
            if all_results.is_empty() {
                writeln!(
                    &mut output_target,
                    "# Dystrail Logic Test Results\n\n_No scenarios executed._"
                )?;
            } else {
                logic::reports::generate_markdown_report(&mut output_target, &all_results)?;
            }
        }
        "csv" => {
            if let Some(records) = playability_records.as_ref() {
                logic::reports::generate_csv_report(&mut output_target, records)?;
            } else {
                writeln!(&mut output_target, "[]")?;
            }
        }
        _ => {
            let duration = start_time.elapsed();
            if all_results.is_empty() {
                writeln!(&mut output_target, "No logic scenarios executed.")?;
            } else if let Some(aggregates) = playability_aggregates.as_ref() {
                logic::reports::generate_console_report(
                    &mut output_target,
                    &all_results,
                    aggregates,
                    duration,
                )?;
            } else {
                writeln!(&mut output_target, "Playability data unavailable.")?;
            }
        }
    }

    if let Some(aggregates) = playability_aggregates.as_ref() {
        let record_slice = playability_records.as_deref().unwrap_or(&[]);
        validate_playability_targets(aggregates, record_slice)?;
    }

    let duration = start_time.elapsed();
    writeln!(&mut output_target)?;
    writeln!(&mut output_target, "🏁 Total time: {duration:?}")?;
    output_target.flush_inner()?;

    // Exit with error code if any tests failed
    let failed_tests = all_results.iter().any(|r| !r.passed);
    if failed_tests {
        std::process::exit(1);
    }

    Ok(())
}
enum OutputTarget {
    Stdout(BufWriter<std::io::Stdout>),
    File(BufWriter<File>),
}

impl OutputTarget {
    fn new(path: Option<PathBuf>) -> Result<Self> {
        if let Some(path) = path {
            let file = File::create(&path)
                .with_context(|| format!("failed to create {}", path.display()))?;
            Ok(Self::File(BufWriter::new(file)))
        } else {
            Ok(Self::Stdout(BufWriter::new(stdout())))
        }
    }

    fn writer(&mut self) -> &mut dyn Write {
        match self {
            Self::Stdout(w) => w,
            Self::File(w) => w,
        }
    }

    fn flush_inner(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdout(w) => w.flush(),
            Self::File(w) => w.flush(),
        }
    }
}

impl Write for OutputTarget {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_inner()
    }
}
