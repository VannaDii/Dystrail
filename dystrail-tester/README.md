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
- `--scenarios smoke,combat,travel` - Scenarios to run (default: smoke)
- `--seeds 1337,1338` - Seeds for deterministic testing (default: 1337)
- `--verbose` - Detailed output

### Logic Mode Options

- `--iterations 10` - Iterations per scenario (default: 10)
- `--report console|json|markdown` - Output format (default: console)

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
cargo run -p dystrail-tester -- --mode both --scenarios smoke,combat --browsers chrome,firefox

# Performance regression testing
cargo run -p dystrail-tester -- --mode logic --iterations 100 --seeds 1337,7331,31337
```

### CI/CD Pipeline

```bash
# Logic tests in CI (fast)
cargo run -p dystrail-tester -- --mode logic --scenarios smoke,combat,travel --iterations 20

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

Create new test scenarios in `src/scenario/`. Each scenario implements both browser and logic testing:

```rust
// src/scenario/combat.rs
use anyhow::Result;
use thirtyfour::prelude::*;
use super::{BrowserScenario, CombinedScenario, ScenarioCtx, TestScenario};

pub struct Combat;

#[async_trait::async_trait]
impl BrowserScenario for Combat {
    async fn run_browser(&self, driver: &WebDriver, ctx: &ScenarioCtx<'_>) -> Result<()> {
        // Browser-specific test logic
        driver.goto(&ctx.base_url).await?;
        ctx.bridge.ensure_available().await?;
        // ... test combat scenarios via browser
        Ok(())
    }
}

impl CombinedScenario for Combat {
    fn as_logic_scenario(&self) -> Option<TestScenario> {
        Some(TestScenario {
            name: "Combat System".to_string(),
            description: "Test combat mechanics and damage calculations".to_string(),
            setup: Some(|game_state| {
                // Set up combat scenario
                game_state.stats.hp = 5;
            }),
            test_fn: |game_state| {
                // Pure logic testing
                anyhow::ensure!(game_state.stats.hp > 0, "Player should survive");
                Ok(())
            },
        })
    }
}
```

Then register it in `src/scenario/mod.rs`:

```rust
pub fn get_scenario(name: &str) -> Option<Box<dyn CombinedScenario + Send + Sync>> {
    match name.to_lowercase().as_str() {
        "smoke" => Some(Box::new(smoke::Smoke)),
        "combat" => Some(Box::new(combat::Combat)),  // Add this line
        _ => None,
    }
}
```

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
