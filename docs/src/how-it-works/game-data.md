# Game data and builds

The versioned JSON files under [`dystrail-web/static/assets/data/`](https://github.com/VannaDii/Dystrail/tree/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-web/static/assets/data) define much of the game's content and tuning. The game's images and other media are separate assets under `dystrail-web/static/`.

| Source | What it describes |
| --- | --- |
| `journey/classic.json`, `journey/deep.json` | Base journey rules for the two modes. |
| `journey/overlays/*.json` | Strategy tuning applied to a base journey configuration. |
| `game.json`, `personas.json`, `store.json` | Encounters, character data, and store items. |
| `pacing.json`, `weather.json`, `vehicle.json`, `camp.json`, `crossings.json` | Day choices and conditions, repairs, rest, and crossings. |
| `boss.json`, `result.json`, `endgame.json`, `exec_orders.json` | Hearing, score display, final approach, and policy events. |

These files are **source data**, not a live settings panel. Many are compiled into the Rust/WebAssembly game with `include_str!`; the published site also serves copies of the JSON. Editing a served JSON file alone does not reliably change the running game's rules. Change the source, build the game, run the relevant validation, and publish the new build to make a variant.

The public [Classic journey JSON](https://dystrail.com/play/static/assets/data/journey/classic.json) is one example of a served source file. For how the game combines base and overlay settings, inspect the [journey loader](https://github.com/VannaDii/Dystrail/blob/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-game/src/journey/mod.rs). For compiled web data, inspect the [web loader](https://github.com/VannaDii/Dystrail/blob/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-web/src/game.rs).
