use anyhow::Result;
use thirtyfour::prelude::*;

use crate::bridge::TestBridge;
use dystrail_web::game::GameState;

pub mod smoke;

#[derive(Debug, Clone)]
pub struct ScenarioCtx<'a> {
    pub base_url: String,
    pub seed: i64,
    pub bridge: TestBridge<'a>,
    pub verbose: bool,
}

// Logic test scenario
pub struct TestScenario {
    pub name: String,
    pub setup: Option<fn(&mut GameState)>,
    pub test_fn: fn(&mut GameState) -> Result<()>,
}

// Browser test scenario
#[async_trait::async_trait]
pub trait BrowserScenario {
    async fn run_browser(&self, driver: &WebDriver, ctx: &ScenarioCtx<'_>) -> Result<()>;
}

// Combined scenario that can run both logic and browser tests
pub trait CombinedScenario: BrowserScenario {
    fn as_logic_scenario(&self) -> Option<TestScenario>;
}

pub fn get_scenario(name: &str) -> Option<Box<dyn CombinedScenario + Send + Sync>> {
    match name.to_lowercase().as_str() {
        "smoke" => Some(Box::new(smoke::Smoke)),
        _ => None,
    }
}
