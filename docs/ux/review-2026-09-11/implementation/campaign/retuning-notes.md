# Campaign retuning investigation — September 12, 2026

The user requires the existing tests and experience targets to remain authoritative. The prior suggestion to revise them was rejected.

## First mechanism: repairs do not reduce wear

The breakdown probability multiplies its base by `1 + beta × vehicle.wear`. Every failure adds five or six wear points. Explicit repairs restored vehicle health by eight points (three for the radio work exchange), but did not reduce wear at all. A vehicle could therefore display full condition while retaining the growing probability of another breakdown. The recorded Classic/Balanced seed 1370 suffered 31 breakdowns and took 22 camp actions in 151 days.

The first candidate reduces wear by the same amount as its explicit durability repair. It retains part, cash, supply, sanity, morale and time costs; a repeated repair request cannot apply another benefit. Focused repair tests passed. In the next 8,000-run measurement, Classic/Balanced breakdowns fell to 3.6 per run; the campaign still failed the existing stopping-time and difficulty checks. No campaign acceptance threshold or player strategy has been changed.

## Further investigation

Fractional daily policy losses currently round independently on every day. In particular, the Balanced overlay configures 0.12 health decay, which rounds to zero each time. Check whether a persisted fractional remainder is needed so small ongoing losses are applied over time without losing them or inventing damage.

## Daily rule ordering

The fractional-remainder candidate alone did not change campaign results: both browser and simulator initialized the day while calculating pace, before the controller reached its daily-effects branch. That branch therefore skipped the configured policy costs. The next candidate installs all policy configuration before the first action and applies daily effects inside the once-per-day initialization. A save/reload and repeated-initialization regression covers the real call order.

## Modern travel requirement, added September 12

The user's new target is Steady at roughly 100–200 road miles/day, Heated at 200–300, and Blitz at 300+. Use nominal clear-road distances of 150, 250 and 350 before weather, vehicle condition, illness and interruptions. Account for the route's actual road miles, not just its normalized simulation distance.

The user explicitly confirmed that condition and encounter probabilities and stat balance must be retuned together to satisfy the pinned experience checks and automated player expectations. Preserve resource safety, stopping-time, encounter variety, reach/win/survival and failure-distribution requirements. Only the older 10–20-mile/day and multi-month trip-duration expectations conflict with the newly requested units and may be replaced with expectations derived from the modern pace targets. Record those changes separately from gameplay tuning. Do not alter automated strategies to manufacture passing results. No retuning candidate is published until it is validated.

## Hourly travel supersedes fixed daily mileage

The user then replaced the fixed mileage approach: Steady should cap at 60 MPH, with higher caps for Heated and Blitz, and distance must follow hours available after all other activities. Current working defaults are 60/70/80 MPH with an eight-hour driving window, 08:00–16:00. A driving step covers at most one hour and ends sooner at a town. Foraging, repairs, care and town work consume their actual hours from the same clock. Daily weather, food, policy and news-diet effects apply once per day; hourly hazards compound to their configured daily risk. Daily mileage caps of 150/250/350 are superseded, not delivered requirements.

The fixed-mile candidates are retained as diagnostic measurements only. The latest visual preview remains the verified 7050e40f35ec177a2368 build. Its full 186-case Chrome desktop/phone regression completed with exit 0 in 5.7 minutes; hourly mechanics are not in that preview yet.
