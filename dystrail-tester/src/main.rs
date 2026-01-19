mod browser;
mod common;
mod logic;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use browser::{BrowserConfig, BrowserKind, TestBridge, new_session};
use common::scenario::{ScenarioCtx, get_scenario, list_scenarios};
use common::{artifacts_dir, capture_artifacts, split_csv};
use logic::{
    GameTester, LogicTester, PlayabilityAggregate, PlayabilityRecord, SeedInfo, TesterAssets,
    aggregate_playability, resolve_seed_inputs, run_playability_analysis,
    validate_playability_targets,
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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if maybe_list_scenarios(&args)? {
        return Ok(());
    }

    announce_banner();

    let playability_iterations = compute_playability_iterations(&args);
    let start_time = Instant::now();
    let scenarios = expand_scenarios(&args.scenarios);
    let seed_tokens = split_csv(&args.seeds);
    let seed_infos = resolve_seed_inputs(&seed_tokens)?;
    let logic_seeds: Vec<u64> = seed_infos.iter().map(|s| s.seed).collect();
    let tester_assets = Arc::new(TesterAssets::load_default());
    let game_tester = GameTester::new(tester_assets, args.verbose);

    let all_results = run_logic_scenarios(&args, &scenarios, &logic_seeds, &game_tester);

    run_browser_scenarios(&args, &scenarios, &seed_infos, &game_tester).await?;

    let (playability_records, playability_aggregates) =
        gather_playability(&args, &game_tester, &seed_infos, playability_iterations)?;

    write_reports(
        &args,
        &all_results,
        playability_records.as_deref(),
        playability_aggregates.as_deref(),
        start_time,
    )?;

    if let Some(aggregates) = playability_aggregates.as_ref() {
        let record_slice = playability_records.as_deref().unwrap_or(&[]);
        validate_playability_targets(aggregates, record_slice)?;
    }

    if all_results.iter().any(|r| !r.passed) {
        std::process::exit(1);
    }

    Ok(())
}

fn maybe_list_scenarios(args: &Args) -> Result<bool> {
    if !args.list_scenarios {
        return Ok(false);
    }
    let mut output_target = OutputTarget::new(args.output.clone())?;
    writeln!(output_target.writer(), "Available scenarios:")?;
    for (key, description) in list_scenarios() {
        writeln!(output_target.writer(), "  {key:25} - {description}")?;
    }
    output_target.flush_inner()?;
    Ok(true)
}

fn announce_banner() {
    println!("{}", "🎮 Dystrail Automated Tester".bright_cyan().bold());
    println!("{}", "================================".cyan());
}

fn compute_playability_iterations(args: &Args) -> usize {
    if args.acceptance {
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
    }
}

fn expand_scenarios(scenarios_arg: &str) -> Vec<String> {
    let mut scenarios = split_csv(scenarios_arg);
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
    scenarios
}

fn parse_browser_kind(name: &str) -> Option<BrowserKind> {
    match name {
        "chrome" => Some(BrowserKind::Chrome),
        "edge" => Some(BrowserKind::Edge),
        "firefox" => Some(BrowserKind::Firefox),
        "safari" => Some(BrowserKind::Safari),
        _ => None,
    }
}

fn build_browser_config(args: &Args) -> BrowserConfig {
    BrowserConfig {
        headless: args.headless.is_headless(),
        implicit_wait_secs: 3,
        remote_hub: args.hub.clone(),
    }
}

fn browser_label(kind: BrowserKind) -> String {
    format!("{kind:?}").to_lowercase()
}

fn scenario_artifacts_dir(args: &Args, kind: BrowserKind, scenario: &str, seed: u64) -> String {
    let label = browser_label(kind);
    artifacts_dir(&args.artifacts_dir, &label, scenario, seed)
}

fn run_logic_scenarios(
    args: &Args,
    scenarios: &[String],
    logic_seeds: &[u64],
    game_tester: &GameTester,
) -> Vec<logic::ScenarioResult> {
    let mut results: Vec<logic::ScenarioResult> = Vec::new();
    if !matches!(args.mode, TestMode::Logic | TestMode::Both) {
        return results;
    }

    println!("{}", "🧠 Running Logic Tests".bright_yellow().bold());
    println!("{}", "-".repeat(30).yellow());

    let logic_tester = LogicTester::new(game_tester.clone());

    for scenario_name in scenarios {
        if let Some(combined_scenario) = get_scenario(scenario_name, game_tester) {
            if let Some(logic_scenario) = combined_scenario.as_logic_scenario() {
                let scenario_results =
                    logic_tester.run_scenario(&logic_scenario, logic_seeds, args.iterations);
                results.extend(scenario_results);
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

    results
}

async fn run_browser_scenarios(
    args: &Args,
    scenarios: &[String],
    seed_infos: &[SeedInfo],
    game_tester: &GameTester,
) -> Result<()> {
    if !matches!(args.mode, TestMode::Browser | TestMode::Both) {
        return Ok(());
    }

    println!("{}", "🌐 Running Browser Tests".bright_blue().bold());
    println!("{}", "-".repeat(30).blue());

    let browsers = split_csv(&args.browsers);

    for browser_name in browsers {
        let Some(kind) = parse_browser_kind(&browser_name) else {
            eprintln!("⚠️  Unknown browser: {}", browser_name.yellow());
            continue;
        };

        let cfg = build_browser_config(args);

        let driver = match new_session(kind, &cfg).await {
            Ok(d) => d,
            Err(e) => {
                eprintln!("❌ Could not start {kind:?}: {e}");
                continue;
            }
        };

        run_browser_scenarios_for_driver(args, scenarios, seed_infos, game_tester, kind, &driver)
            .await;
        let _ = driver.quit().await;
    }

    Ok(())
}

async fn run_browser_scenarios_for_driver(
    args: &Args,
    scenarios: &[String],
    seed_infos: &[SeedInfo],
    game_tester: &GameTester,
    kind: BrowserKind,
    driver: &thirtyfour::WebDriver,
) {
    for scenario_name in scenarios {
        if let Some(scenario) = get_scenario(scenario_name, game_tester) {
            for seed_info in seed_infos {
                let bridge = TestBridge::new(driver);
                let ctx = ScenarioCtx {
                    base_url: args.base_url.clone(),
                    seed: seed_info.seed,
                    bridge,
                    verbose: args.verbose,
                };

                let label = browser_label(kind);
                let dir = scenario_artifacts_dir(args, kind, scenario_name, seed_info.seed);

                let scenario_start = Instant::now();
                match scenario.run_browser(driver, &ctx).await {
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
                        let _ = capture_artifacts(driver, &dir, &e).await;
                    }
                }
            }
        }
    }
}

type PlayabilitySummary = (
    Option<Vec<PlayabilityRecord>>,
    Option<Vec<PlayabilityAggregate>>,
);

fn gather_playability(
    args: &Args,
    game_tester: &GameTester,
    seed_infos: &[SeedInfo],
    playability_iterations: usize,
) -> Result<PlayabilitySummary> {
    let mut playability_records: Option<Vec<PlayabilityRecord>> = None;
    let mut playability_aggregates: Option<Vec<PlayabilityAggregate>> = None;
    let require_playability = matches!(args.report.as_str(), "console" | "csv")
        || matches!(args.mode, TestMode::Logic | TestMode::Both);

    if require_playability {
        let playability =
            run_playability_analysis(game_tester, seed_infos, playability_iterations)?;
        playability_aggregates = Some(aggregate_playability(&playability));
        playability_records = Some(playability);
    }

    Ok((playability_records, playability_aggregates))
}

fn write_reports(
    args: &Args,
    results: &[logic::ScenarioResult],
    playability_records: Option<&[PlayabilityRecord]>,
    playability_aggregates: Option<&[PlayabilityAggregate]>,
    start_time: Instant,
) -> Result<()> {
    let mut output_target = OutputTarget::new(args.output.clone())?;

    match args.report.as_str() {
        "json" => {
            if results.is_empty() {
                writeln!(&mut output_target, "[]")?;
            } else {
                logic::reports::generate_json_report(&mut output_target, results)?;
            }
        }
        "markdown" => {
            if results.is_empty() {
                writeln!(
                    &mut output_target,
                    "# Dystrail Logic Test Results\n\n_No scenarios executed._"
                )?;
            } else {
                logic::reports::generate_markdown_report(&mut output_target, results)?;
            }
        }
        "csv" => {
            if let Some(records) = playability_records {
                logic::reports::generate_csv_report(&mut output_target, records)?;
            } else {
                writeln!(&mut output_target, "[]")?;
            }
        }
        _ => {
            let duration = start_time.elapsed();
            if results.is_empty() {
                writeln!(&mut output_target, "No logic scenarios executed.")?;
            } else if let Some(aggregates) = playability_aggregates {
                logic::reports::generate_console_report(
                    &mut output_target,
                    results,
                    aggregates,
                    duration,
                )?;
            } else {
                writeln!(&mut output_target, "Playability data unavailable.")?;
            }
        }
    }

    let duration = start_time.elapsed();
    writeln!(&mut output_target)?;
    writeln!(&mut output_target, "🏁 Total time: {duration:?}")?;
    output_target.flush_inner()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::game_tester::{
        BossOutcomeFlags, EndgameStatus, PlayabilityMetrics, RunMilestones,
    };
    use crate::logic::{
        GameTester, GameplayStrategy, PlayabilityAggregate, PlayabilityRecord, ScenarioResult,
        SeedInfo, TesterAssets,
    };
    use std::io::Write;
    use std::sync::Arc;
    use std::time::Duration;

    fn base_args() -> Args {
        Args {
            mode: TestMode::Logic,
            scenarios: "smoke".to_string(),
            list_scenarios: false,
            seeds: "1337".to_string(),
            iterations: 1,
            acceptance: false,
            report: "json".to_string(),
            verbose: false,
            output: None,
            browsers: "chrome".to_string(),
            base_url: "http://localhost:5173/?test=1".to_string(),
            artifacts_dir: "target/test-artifacts".to_string(),
            hub: None,
            headless: HeadlessMode::Headless,
        }
    }

    fn sample_metrics() -> PlayabilityMetrics {
        let mut metrics = PlayabilityMetrics::default();
        metrics.days_survived = 5;
        metrics.ending_type = "Victory".to_string();
        metrics.ending_cause = "None".to_string();
        metrics.miles_traveled = 120.0;
        metrics.avg_miles_per_day = 12.0;
        metrics.travel_ratio = 0.8;
        metrics.unique_per_20_days = 1.2;
        metrics.rotation_events = 2;
        metrics.milestones = RunMilestones {
            reached_2000_by_day150: true,
            survived: true,
        };
        metrics.boss = BossOutcomeFlags {
            reached: true,
            won: false,
        };
        metrics.endgame = EndgameStatus {
            active: true,
            field_repair_used: false,
        };
        metrics
    }

    fn sample_record() -> PlayabilityRecord {
        PlayabilityRecord {
            scenario_name: "Smoke".to_string(),
            mode: dystrail_game::GameMode::Classic,
            strategy: GameplayStrategy::Balanced,
            seed_code: "CL-ORANGE42".to_string(),
            seed_value: 42,
            metrics: sample_metrics(),
        }
    }

    fn sample_aggregate() -> PlayabilityAggregate {
        PlayabilityAggregate {
            scenario_name: "Smoke".to_string(),
            mode: dystrail_game::GameMode::Classic,
            strategy: GameplayStrategy::Balanced,
            iterations: 1,
            mean_days: 5.0,
            std_days: 0.0,
            mean_miles: 120.0,
            std_miles: 0.0,
            mean_avg_mpd: 12.0,
            boss_reach_pct: 0.4,
            boss_win_pct: 0.2,
            pants_failure_pct: 0.1,
            mean_travel_ratio: 0.8,
            mean_unique_per_20: 1.2,
            mean_rotation_events: 2.0,
            pct_reached_2k_by_150: 0.5,
            min_unique_per_20: 0.8,
            min_travel_ratio: 0.7,
            mean_crossing_events: 1.0,
            crossing_permit_rate: 0.6,
            mean_crossing_bribes: 0.1,
            crossing_bribe_success_rate: 0.2,
            mean_crossing_detours: 0.0,
            crossing_failure_rate: 0.05,
            mean_stop_cap_conversions: 0.0,
            endgame_activation_rate: 0.3,
            endgame_field_repair_rate: 0.1,
            mean_endgame_cooldown: 2.0,
            survival_rate: 0.9,
            failure_vehicle_pct: 0.0,
            failure_sanity_pct: 0.0,
            failure_exposure_pct: 0.0,
            failure_crossing_pct: 0.0,
        }
    }

    fn sample_result(passed: bool) -> ScenarioResult {
        ScenarioResult {
            scenario_name: "Smoke".to_string(),
            passed,
            iterations_run: 3,
            successful_iterations: if passed { 3 } else { 2 },
            failures: if passed {
                Vec::new()
            } else {
                vec!["failure".to_string()]
            },
            average_duration: Duration::from_millis(10),
            performance_data: vec![Duration::from_millis(10)],
        }
    }

    #[test]
    fn computes_playability_iterations_for_acceptance() {
        let mut args = base_args();
        args.acceptance = true;
        args.iterations = 10;
        assert_eq!(compute_playability_iterations(&args), 100);
        args.iterations = 150;
        assert_eq!(compute_playability_iterations(&args), 150);
    }

    #[test]
    fn expands_all_scenarios_keyword() {
        let expanded = expand_scenarios("all,smoke");
        assert!(expanded.contains(&"smoke".to_string()));
        assert!(expanded.contains(&"vehicle-system".to_string()));
    }

    #[test]
    fn expand_scenarios_without_all_preserves_order() {
        let expanded = expand_scenarios("smoke,real-game");
        assert_eq!(expanded, vec!["smoke".to_string(), "real-game".to_string()]);
    }

    #[test]
    fn compute_playability_iterations_returns_default_when_disabled() {
        let args = base_args();
        assert_eq!(compute_playability_iterations(&args), 1);
    }

    #[test]
    fn run_logic_scenarios_skips_when_not_enabled() {
        let assets = Arc::new(TesterAssets::load_default());
        let tester = GameTester::new(assets, false);
        let args = Args {
            mode: TestMode::Browser,
            ..base_args()
        };
        let results = run_logic_scenarios(&args, &["smoke".to_string()], &[42], &tester);
        assert!(results.is_empty());
    }

    #[test]
    fn gather_playability_returns_none_when_disabled() {
        let assets = Arc::new(TesterAssets::load_default());
        let tester = GameTester::new(assets, false);
        let args = Args {
            mode: TestMode::Browser,
            report: "json".to_string(),
            ..base_args()
        };
        let seeds = vec![SeedInfo::from_numeric(42)];
        let (records, aggregates) = gather_playability(&args, &tester, &seeds, 1).unwrap();
        assert!(records.is_none());
        assert!(aggregates.is_none());
    }

    #[test]
    fn write_reports_emits_json_output() {
        let args = Args {
            report: "json".to_string(),
            ..base_args()
        };
        let results = vec![];
        let start = Instant::now();
        let temp = std::env::temp_dir().join("dystrail-test-report.json");
        let args = Args {
            output: Some(temp.clone()),
            ..args
        };
        write_reports(&args, &results, None, None, start).unwrap();
        let content = std::fs::read_to_string(temp).unwrap();
        assert!(content.contains("[]"));
    }

    #[test]
    fn maybe_list_scenarios_writes_output() {
        let temp = std::env::temp_dir().join("dystrail-scenarios.txt");
        let args = Args {
            list_scenarios: true,
            output: Some(temp.clone()),
            ..base_args()
        };
        assert!(maybe_list_scenarios(&args).unwrap());
        let content = std::fs::read_to_string(temp).unwrap();
        assert!(content.contains("Available scenarios"));
    }

    #[test]
    fn maybe_list_scenarios_returns_false_when_disabled() {
        let args = base_args();
        assert!(!maybe_list_scenarios(&args).unwrap());
    }

    #[test]
    fn write_reports_markdown_empty_results() {
        let temp = std::env::temp_dir().join("dystrail-report.md");
        let args = Args {
            report: "markdown".to_string(),
            output: Some(temp.clone()),
            ..base_args()
        };
        write_reports(&args, &[], None, None, Instant::now()).unwrap();
        let content = std::fs::read_to_string(temp).unwrap();
        assert!(content.contains("No scenarios executed"));
    }

    #[test]
    fn write_reports_emits_markdown_report() {
        let temp = std::env::temp_dir().join("dystrail-report-full.md");
        let args = Args {
            report: "markdown".to_string(),
            output: Some(temp.clone()),
            ..base_args()
        };
        write_reports(&args, &[sample_result(true)], None, None, Instant::now()).unwrap();
        let content = std::fs::read_to_string(temp).unwrap();
        assert!(content.contains("# Dystrail Logic Test Results"));
        assert!(content.contains("Smoke"));
    }

    #[test]
    fn write_reports_emits_json_for_results() {
        let temp = std::env::temp_dir().join("dystrail-report-full.json");
        let args = Args {
            report: "json".to_string(),
            output: Some(temp.clone()),
            ..base_args()
        };
        write_reports(&args, &[sample_result(true)], None, None, Instant::now()).unwrap();
        let content = std::fs::read_to_string(temp).unwrap();
        assert!(content.contains("scenario_name"));
    }

    #[test]
    fn write_reports_emits_csv_report() {
        let temp = std::env::temp_dir().join("dystrail-report.csv");
        let args = Args {
            report: "csv".to_string(),
            output: Some(temp.clone()),
            ..base_args()
        };
        let record = sample_record();
        write_reports(&args, &[], Some(&[record]), None, Instant::now()).unwrap();
        let content = std::fs::read_to_string(temp).unwrap();
        assert!(content.contains("scenario,mode,strategy"));
        assert!(content.contains("Smoke"));
    }

    #[test]
    fn write_reports_emits_console_report_with_playability() {
        let temp = std::env::temp_dir().join("dystrail-report-console.txt");
        let args = Args {
            report: "console".to_string(),
            output: Some(temp.clone()),
            ..base_args()
        };
        let result = sample_result(true);
        let aggregate = sample_aggregate();
        write_reports(&args, &[result], None, Some(&[aggregate]), Instant::now()).unwrap();
        let content = std::fs::read_to_string(temp).unwrap();
        assert!(content.contains("Playability Summary"));
    }

    #[test]
    fn write_reports_console_without_playability() {
        let temp = std::env::temp_dir().join("dystrail-report.txt");
        let args = Args {
            report: "console".to_string(),
            output: Some(temp.clone()),
            ..base_args()
        };
        let result = logic::ScenarioResult {
            scenario_name: "smoke".to_string(),
            passed: true,
            iterations_run: 1,
            successful_iterations: 1,
            failures: Vec::new(),
            average_duration: Duration::ZERO,
            performance_data: Vec::new(),
        };
        write_reports(&args, &[result], None, None, Instant::now()).unwrap();
        let content = std::fs::read_to_string(temp).unwrap();
        assert!(content.contains("Playability data unavailable"));
    }

    #[test]
    fn output_target_stdout_writes() {
        let mut target = OutputTarget::new(None).unwrap();
        target.write_all(b"ok").unwrap();
        target.flush().unwrap();
    }

    #[test]
    fn parse_browser_kind_handles_known_and_unknown() {
        assert!(matches!(
            parse_browser_kind("chrome"),
            Some(BrowserKind::Chrome)
        ));
        assert!(matches!(
            parse_browser_kind("edge"),
            Some(BrowserKind::Edge)
        ));
        assert!(parse_browser_kind("unknown").is_none());
    }

    #[test]
    fn build_browser_config_respects_headless_and_hub() {
        let mut args = base_args();
        args.headless = HeadlessMode::Windowed;
        args.hub = Some("http://remote.example".to_string());
        let cfg = build_browser_config(&args);
        assert!(!cfg.headless);
        assert_eq!(cfg.remote_hub.as_deref(), Some("http://remote.example"));
    }

    #[test]
    fn scenario_artifacts_dir_includes_seed() {
        let args = base_args();
        let dir = scenario_artifacts_dir(&args, BrowserKind::Chrome, "smoke", 42);
        assert!(dir.contains("smoke/seed-42"));
    }

    #[test]
    fn run_browser_scenarios_skips_when_not_enabled() {
        let assets = Arc::new(TesterAssets::load_default());
        let tester = GameTester::new(assets, false);
        let args = Args {
            mode: TestMode::Logic,
            ..base_args()
        };
        let seeds = vec![SeedInfo::from_numeric(42)];
        tokio_test::block_on(run_browser_scenarios(
            &args,
            &["smoke".to_string()],
            &seeds,
            &tester,
        ))
        .expect("browser scenarios should skip");
    }

    #[test]
    fn run_browser_scenarios_ignores_unknown_browser() {
        let assets = Arc::new(TesterAssets::load_default());
        let tester = GameTester::new(assets, false);
        let args = Args {
            mode: TestMode::Browser,
            browsers: "unknown".to_string(),
            ..base_args()
        };
        let seeds = vec![SeedInfo::from_numeric(42)];
        tokio_test::block_on(run_browser_scenarios(
            &args,
            &["smoke".to_string()],
            &seeds,
            &tester,
        ))
        .expect("unknown browser should be skipped");
    }
}
