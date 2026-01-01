//! Mechanical policy overlays and parity-oriented constants.
//!
//! This module is the home for mechanics-level policy that must be explicit to
//! prevent accidental drift (e.g., Oregon Trail Deluxe 90s parity vs Dystrail
//! legacy behavior). Presentation-layer satire is intentionally out of scope.

pub mod otdeluxe90s;

pub use otdeluxe90s::{
    OtDeluxe90sPolicy, OtDeluxeOccupation, OtDeluxePace, OtDeluxeRations, OtDeluxeTrailVariant,
};
