# Rules and replay

## A run is a sequence of decisions

The game stores its mode, run seed, crew, resources, and route progress in game state. Its recovery checkpoint also stores the current screen. Travel and non-travel actions change that state. At day end, the game records whether the day had travel, partial travel, or no travel, along with the distance earned. Camp and delays can therefore cost time without pretending that the van moved.

The run code chooses Classic (`CL-`) or The Deep End (`DP-`) and a seed. The game derives separate random streams for weather, health, travel, events, breakdowns, encounters, crossings, the hearing, trade, and hunting. This keeps one subsystem's random draws from automatically shifting every other subsystem. Replaying the same code, choices, and game version can reproduce the same run. A saved game also stores the state needed to resume mid-run; a code does not.

## Travel and decisions

Before a day starts, the player chooses a pace and information diet. The game combines pace, available driving time, weather, vehicle condition, and active conditions to determine actual progress. The information diet applies a daily sanity effect and changes the chance of finding an extra receipt. Encounters, repairs, crossings, gathering, and town actions can change resources or consume time. The game displays current option costs where the choice is offered and gives an action receipt afterward.

The route target and the hearing's distance threshold are separate settings. The route the player sees is set by the resolved journey configuration; the hearing computes starting odds from the current journey score and its own configuration. A journey score includes remaining supplies, health, morale, credibility, allies, days, resolved encounters, and kept receipts, with a breakdown penalty. The hearing then spends sanity during up to three rounds, adjusts the starting odds by completed-round influence, and draws a final vote only when needed. Its resolved report is saved before the presentation plays.

For the exact implementation, see the [journey state](https://github.com/VannaDii/Dystrail/blob/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-game/src/state.rs), [random streams and configuration](https://github.com/VannaDii/Dystrail/blob/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-game/src/journey/mod.rs), and [hearing resolution](https://github.com/VannaDii/Dystrail/blob/0fd8b1d0227239a43b61110d62fe21c851be803e/dystrail-game/src/boss.rs) in the game revision this guide describes.
