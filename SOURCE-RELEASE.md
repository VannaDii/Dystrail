# Satire content release

Game revision: `644ac8f90658df167f1d`. Previous production revision: `4a35a2024033bd2ce615`.

Canonical source: [`release/complete-game-source-satire-2026-09-14`](https://github.com/VannaDii/Dystrail/tree/release/complete-game-source-satire-2026-09-14), commit [`30652f13b5c9b6a28e31487eea5885a305e24ebc`](https://github.com/VannaDii/Dystrail/commit/30652f13b5c9b6a28e31487eea5885a305e24ebc). This branch packages the Pages artifact; its legacy source directories are not the canonical game source.

[Source CI](https://github.com/VannaDii/Dystrail/actions/runs/34811445713) produced the exact artifact in `release-site/play`. Publication requires successful completion of every source CI gate.

The release imports 177 A-version workshop packages into existing gameplay: 65 encounters with 187 ordered choices, 44 intermediate town conversations, and the care, departures, crossings, orders, repairs, activities, conditions, openings, hearing and ending prose supported by the current game. English, Spanish, Italian and Arabic are included; other language interfaces retain English fallback for new narrative. B/C variants, twelve proposed families, new mechanics and new art are outside this release.

The engine, choice effects, costs, probabilities and save schema are unchanged. All 59 protected website files and 43 older hashed game assets are retained. The artifact contains 119 offline assets. A complete saved journey survived the update, offline reload and a closed-browser offline restart with every cache integrity hash checked.

Source snapshot and integration decisions are recorded under `docs/release/satire-content-2026-09-14` in the source branch. Artifact verification is recorded under the same path here. The deployment procedure temporarily permits only this exact release branch, then restores the existing main-only Pages policy.
