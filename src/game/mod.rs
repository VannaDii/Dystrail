pub mod boss;
pub mod data;
pub mod encounters;
pub mod exec_orders;
pub mod pacing;
pub mod personas;
pub mod seed;
pub mod state;
pub mod store;
pub mod vehicle;
pub mod weather;

pub use personas::{Persona, PersonaMods, PersonaStart};
pub use state::{GameMode, GameState, Region, Stats, Inventory, Spares};
pub use store::{Store, StoreItem, StoreCategory, Cart, CartLine, Grants, calculate_effective_price, calculate_cart_total};
pub use pacing::{PacingConfig, PaceCfg, DietCfg, PacingLimits};
pub use vehicle::{Part, Vehicle, Breakdown, VehicleConfig, breakdown_roll, weighted_pick};
pub use weather::{Weather, WeatherState, WeatherConfig, WeatherEffect, WeatherMitigation, process_daily_weather};
