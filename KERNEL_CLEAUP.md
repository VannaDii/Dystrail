# OTDeluxe Parity Kernel Cleanup Plan

## Summary
Refactor `dystrail-game` into a strict OTDeluxe parity kernel with no satire/content logic in mechanics.
All text/theme/satire moves to `dystrail-web` i18n via stable event IDs + typed payloads.
This initiative is a single large PR with legacy mechanics removed in the same effort and hard save reset.

## Locked Decisions
- Kernel scope: OT-only core
- Compatibility: breaking API allowed
- Structure: new `src/kernel/*` module tree
- Legacy path: remove in same initiative
- Theme contract: stable event IDs + typed payloads
- Delivery: single large PR
- i18n ownership: `dystrail-web` only
- Save migration: hard reset saves

## Target End State
- `dystrail-game` contains deterministic simulation only.
- `dystrail-web` owns all localization/theme/satire copy.
- Kernel emits stable event codes and typed payloads.
- Legacy policy mechanics are removed from core day pipeline.

## Planned Module Layout
- `dystrail-game/src/kernel/mod.rs`
- `dystrail-game/src/kernel/types.rs`
- `dystrail-game/src/kernel/events/{ids,payload,trace}.rs`
- `dystrail-game/src/kernel/phases/*`
- `dystrail-game/src/kernel/systems/*`
- `dystrail-game/src/kernel/policy/otdeluxe90s.rs`

## Public API Direction
- Introduce kernel-first simulation API:
  - `KernelConfig`
  - `KernelState`
  - `KernelTickInput`
  - `KernelTickOutput`
- Replace direct legacy day tick entrypoints with kernel tick orchestration.
- Emit stable event codes and structured payloads from kernel.

## Implementation Order
- [x] Add guardrails + freeze event code schema.
- [x] Extract kernel types and phase boundaries.
  - [x] `kernel/types.rs` introduced and wired.
  - [x] `kernel/events/*` introduced and schema frozen.
  - [x] `kernel/session.rs` extracted from `kernel/mod.rs`.
  - [x] `kernel/phases/mod.rs` introduced as explicit tick boundary.
- [ ] Migrate mechanics out of `state.rs` by phase.
  - [x] Extract OT navigation helper/event-roll functions into `kernel/systems/navigation.rs`.
  - [x] Extract OT affliction probability + affliction-kind roll helpers into `kernel/systems/affliction.rs`.
  - [x] Extract OT affliction duration helper into `kernel/systems/affliction.rs`.
  - [x] Extract OT pace/rations supply math helpers into `kernel/systems/supplies.rs`.
  - [x] Extract OT weather/clothing/affliction/drought health penalties into `kernel/systems/health.rs`.
  - [x] Extract OT health delta composition helper into `kernel/systems/health.rs`.
  - [x] Extract OT snow-speed multiplier helper into `kernel/systems/travel.rs`.
  - [x] Extract OT starting-cash helper into `kernel/systems/economy.rs`.
  - [x] Extract OT disease fatality probability/roll helpers into `kernel/systems/fatality.rs`.
  - [x] Extract OT spare-part mapping/consumption helpers into `kernel/systems/vehicle.rs`.
  - [x] Extract OT occupation mobility-failure multiplier helper into `kernel/systems/vehicle.rs`.
  - [x] Extract OT disease effect application/multiplier sanitizer into `kernel/systems/disease.rs`.
- [ ] Canonicalize OT systems and remove duplicates.
- [ ] Rewire crate exports around kernel API.
- [ ] Move narrative strings fully to `dystrail-web` i18n.
- [ ] Apply save reset/version bump and remove legacy migration logic.
- [ ] Run full validation workflow.

## Testing and Acceptance
- Deterministic seed replay must remain stable.
- Phase order and RNG stream ownership tests must pass.
- Hard-stop invariants (zero travel miles on nav blocks) must hold.
- Event code list and payload schema snapshots must be stable.
- Locale interpolation coverage in `en`, `it`, `es`, `ar` must pass.

## Required Validation Commands
- `just fmt`
- `just lint`
- `just tests`
- `just security`
- `just build-release`
- `just qa`
- `just validate`
