# Game data and builds

The versioned JSON files under [`dystrail-web/static/assets/data/`](https://github.com/VannaDii/Dystrail/tree/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-web/static/assets/data) define much of the game's content and tuning. The game's images and other media are separate assets under `dystrail-web/static/`.

- **Journey rules:** `journey/classic.json` and `journey/deep.json` contain the base rules for each mode. Files in `journey/overlays/` adjust those rules for strategies.
- **People, encounters, and equipment:** `game.json`, `personas.json`, and `store.json` describe encounters, characters, and store items.
- **The road:** `pacing.json`, `weather.json`, `vehicle.json`, `camp.json`, and `crossings.json` define day choices, conditions, repairs, rest, and crossings.
- **D.C. and the result:** `boss.json`, `result.json`, `endgame.json`, and `exec_orders.json` cover the hearing, score display, final approach, and policy events.

These files are **source data**, not a live settings panel. Many are compiled into the Rust/WebAssembly game with `include_str!`; the published site also serves copies of the JSON. Editing a served JSON file alone does not reliably change the running game's rules. Change the source, build the game, run the relevant validation, and publish the new build to make a variant.

The public [Classic journey JSON](https://dystrail.com/play/static/assets/data/journey/classic.json) is one example of a served source file. For how the game combines base and overlay settings, inspect the [journey loader](https://github.com/VannaDii/Dystrail/blob/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-game/src/journey/mod.rs). For compiled web data, inspect the [web loader](https://github.com/VannaDii/Dystrail/blob/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-web/src/game.rs).
