# Dystrail Tester

Advanced QA testing suite for Dystrail with dual testing modes:

1. **Logic Testing** - Fast, pure Rust game logic validation
2. **Browser Testing** - Full end-to-end testing with WebDriver automation

## Why This Design?

- **Canvas-friendly**: DOM-only assertions are brittle for canvas games; we use a JS test bridge for reliable state inspection
- **Deterministic**: Seed RNG and accelerate simulation for consistent, fast tests
- **Dual-mode**: Logic tests catch regressions quickly, browser tests catch integration issues
- **Portable**: Works with Chrome/Edge/Firefox/Safari; optional Grid for CI/CD scale

## Quick Start

### Logic Testing (Fast)

```bash
# Test core game logic with deterministic seeds
cargo run -p dystrail-tester -- --mode logic --scenarios smoke --seeds 1337,1338 --iterations 50
```

#### Automated Play Simulation

Every logic scenario now runs through the shared `SimulationPlan` harness. Policies, setup steps, and assertions live together so both logic and browser test paths stay in sync.

```bash
# Quick smoke across default seeds
cargo run -p dystrail-tester -- --mode logic --scenarios smoke --seeds 1337,9001 --iterations 5

# Full playability sweep (Balanced, Conservative, Aggressive, Resource, MonteCarlo)
cargo run -p dystrail-tester -- --mode logic --seeds 1337 --report csv

# Exercise the long-form conservative policy with verbose logging
cargo run -p dystrail-tester -- --mode logic --scenarios full-game-conservative --seeds 1234 --iterations 3 --verbose

# Stress-test resource and edge-case survival scenarios
cargo run -p dystrail-tester -- --mode logic --scenarios resource-stress,edge-case --seeds 42 --iterations 2
```

Key flags (see `--help` for the complete list):

- `--mode logic|browser|both` — select the execution backend (logic is fastest).
- `--scenarios <names>` — comma-separated identifiers such as `smoke`, `full-game-conservative`, `resource-stress`, or `all`.
- `--seeds <values>` — numeric seeds; share codes are resolved automatically.
- `--iterations <n>` — number of times to repeat each seed/scenario pair.
- `--report console|json|markdown|csv` — switch output formats. CSV emits playability metrics with decision logs.
- `--verbose` — print turn-by-turn decisions with policy rationales for debugging.

Available automated strategies: `Balanced`, `Conservative`, `Aggressive`, `ResourceManager`, and `MonteCarlo` (stochastic search with stable RNG seeding).

Failures emit rich diagnostics—including final stats snapshots and the last three encounter decisions—which helps triage regressions after tweaking JSON content.

### Scenario Catalog (Logic Mode)

| Scenario | Purpose | Highlights |
| --- | --- | --- |
| `smoke` | Basic invariants | HP/supplies/sanity ranges, day ≥ 1 |
| `real-game` | Balanced policy regression | Survives > 0 days, no error endings |
| `conservative-strategy`, `aggressive-strategy`, `resource-manager` | Classic heuristics | Scenario-specific survival/pants limits |
| `full-game-conservative`, `full-game-aggressive`, `full-game-balanced` | 40-day policy runs | Checks breakdowns, encounters, and risk curves |
| `resource-stress`, `edge-case` | Failure pressure tests | Must terminate via resource or pants collapse |
| `deterministic` | Reproducibility guard | Second run must match turn count and stats |
| Catalog scenarios (`basic`, `inventory`, `weather-effects`, …) | Targeted system checks | Share-code round trip, stat clamping, vehicle ops, etc. |

Use `--scenarios all` to execute the entire catalog in one go.

### Browser Testing (Visual)

```bash
# Start your game server with test bridge
trunk serve --open --port 5173 # or any static server

# Start browser drivers (pick what you have)
chromedriver --port=9515        # Chrome
geckodriver --port 4444         # Firefox
msedgedriver --port=17556       # Edge
safaridriver -p 4445            # Safari (macOS only)

# Run browser tests
cargo run -p dystrail-tester -- --mode browser --browsers chrome,firefox --base-url "http://localhost:5173/?test=1"
```

### Both Modes

```bash
cargo run -p dystrail-tester -- --mode both --scenarios smoke --seeds 1337 --browsers chrome
```

## Test Bridge Integration

Your game needs to expose a test bridge when `?test=1` is in the URL:

```javascript
// Add this to your game when in test mode
if (location.search.includes('test=1')) {
  window.__dystrailTest = {
    seed: (n) => {
      // Set your game's RNG seed
      game.rng.seed(n);
    },
    speed: (mult) => {
      // Accelerate game simulation
      game.tickRate *= mult;
    },
    state: () => ({
      screen: game.currentScreen,
      hp: game.player.hp,
      day: game.day,
      pos: game.player.position,
    }),
    click: (x, y) => {
      // Simulate canvas click
      game.handleClick(x, y);
    },
    key: (key) => {
      // Simulate keyboard input
      game.handleKey(key);
    },
  };
}
```

## Command Line Options

### Core Options

- `--mode logic|browser|both` - Test mode (default: logic)
- `--scenarios smoke,full-game-conservative,resource-stress` - Comma-separated scenario names (default: smoke)
- `--seeds 1337,1338` - Seeds for deterministic testing (default: 1337)
- `--verbose` - Detailed output

### Logic Mode Options

- `--iterations 10` - Iterations per scenario (default: 10)
- `--report console|json|markdown|csv` - Output format (default: console)

### Browser Mode Options

- `--browsers chrome,firefox,edge,safari` - Browsers to test (default: chrome)
- `--base-url http://localhost:5173/?test=1` - Game URL with test bridge
- `--artifacts-dir target/test-artifacts` - Screenshot/error dump location
- `--hub http://selenium-grid:4444` - Selenium Grid URL (optional)
- `--headless` - Run browsers headlessly (default: true)

## Examples

### Development Workflow

```bash
# Quick smoke test during development
cargo run -p dystrail-tester -- --mode logic --verbose

# Full validation before commit
cargo run -p dystrail-tester -- --mode both --scenarios smoke,full-game-conservative --browsers chrome,firefox

# Performance regression testing
cargo run -p dystrail-tester -- --mode logic --iterations 100 --seeds 1337,7331,31337
```

### CI/CD Pipeline

```bash
# Logic tests in CI (fast)
cargo run -p dystrail-tester -- --mode logic --scenarios smoke,full-game-conservative,resource-stress --iterations 20

# Browser tests in CI with Selenium Grid
export SELENIUM_HUB=http://selenium-grid:4444
cargo run -p dystrail-tester -- --mode browser --hub $SELENIUM_HUB --browsers chrome,firefox
```

## Artifacts

When browser tests fail, artifacts are saved to `artifacts-dir/{browser}/{scenario}/seed-{n}/timestamp/`:

- `screenshot.png` - Full page screenshot at failure
- `dom.html` - Page source (useful even for canvas games)
- `state.json` - Game state via test bridge
- `error.txt` - Full error chain

## Adding New Scenarios

1. **Create a simulation-backed scenario** in `src/common/scenario/`.

    ```rust
    pub fn combat_scenario() -> SimulationScenario {
        SimulationScenario::new(
            "Combat System",
            SimulationPlan::new(GameMode::Classic, GameplayStrategy::Aggressive)
                .with_setup(|state| {
                    state.stats.hp = 5;
                    state.stats.supplies = 8;
                })
                .with_expectation(|summary| {
                    anyhow::ensure!(
                        !summary.turns.is_empty(),
                        "Combat scenario produced no turns"
                    );
                    anyhow::ensure!(
                        summary.metrics.final_hp > 0,
                        "Player should survive the combat simulation"
                    );
                    Ok(())
                }),
        )
    }
    ```

2. **Wire it into the dispatcher** inside `src/common/scenario/mod.rs::get_scenario` so both logic and browser modes recognise the new identifier.

3. **(Optional) Add browser automation** by implementing `BrowserScenario` for a struct in the same module if you need DOM/bridge coverage (see `smoke.rs` for reference).

4. **Keep expectations meaningful**—run for a realistic duration (`with_max_days`), and assert the metrics that matter (resource trends, encounter pacing, deterministic replay, etc.).

## Browser Driver Setup

### Chrome

```bash
# Install ChromeDriver
brew install chromedriver  # macOS
# or download from https://chromedriver.chromium.org/

# Run
chromedriver --port=9515
```

### Firefox

```bash
# Install GeckoDriver
brew install geckodriver  # macOS
# or download from https://github.com/mozilla/geckodriver/releases

# Run
geckodriver --port 4444
```

### Edge

```bash
# Download from https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/
msedgedriver --port=17556
```

### Safari (macOS only)

```bash
# Enable automation (one time)
sudo safaridriver --enable

# Run
safaridriver -p 4445
```

## Selenium Grid (CI/CD)

For scalable testing in CI/CD pipelines:

```yaml
# docker-compose.yml
version: '3'
services:
  selenium-hub:
    image: selenium/hub:latest
    ports:
      - '4444:4444'

  chrome:
    image: selenium/node-chrome:latest
    depends_on:
      - selenium-hub
    environment:
      - HUB_HOST=selenium-hub

  firefox:
    image: selenium/node-firefox:latest
    depends_on:
      - selenium-hub
    environment:
      - HUB_HOST=selenium-hub
```

```bash
docker-compose up -d
export SELENIUM_HUB=http://localhost:4444
cargo run -p dystrail-tester -- --mode browser --hub $SELENIUM_HUB
```

## Performance Tips

- **Logic mode**: Use high iteration counts (100+) for performance regression detection
- **Browser mode**: Use `--headless` for faster execution in CI
- **Seeds**: Use consistent seeds across test runs for deterministic results
- **Scenarios**: Start with smoke tests, add specific scenario tests as needed

## Troubleshooting

### "Bridge not available" error

- Ensure your game URL includes `?test=1`
- Verify the test bridge is properly exposed in your JavaScript
- Check browser console for JavaScript errors

### Browser driver connection failed

- Verify the correct driver is installed and running
- Check port conflicts (each browser uses different default ports)
- Try running with `--verbose` for detailed error information

### Tests are flaky

- Ensure deterministic seeding is working (`ctx.bridge.seed()`)
- Add appropriate waits for async operations
- Use the test bridge instead of DOM queries where possible
