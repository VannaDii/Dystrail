# Gathering and work scenes

Four existing activity families now use all twelve workshop A/B/C variants: foraging, gleaning, pantry work, and paid work. Three pixel-art atlases provide twenty-four frames, with a separate offered and completed frame for each variant. The original generated sources are retained; explicit SVG bounds and clipping exclude neighboring cells without modifying the images.

An offer is selected from the presentation seed and saved when its menu is shown. Roadside offers belong to the region and gathering cycle; work offers belong to the route and town stop. Opening a menu, reloading, and importing a save do not consume simulation randomness. Older saves keep A where an offer has already been selected. The completed report freezes the offer’s identity before the activity changes the day or cooldown.

The existing engine remains responsible for availability, duration, rewards, resource costs, gathering cooldown, one shift per stop, and overnight settlement. Artwork shows completed work only after `perform_activity` succeeds. The same guard prevents rapid repeat clicks. The completed scene uses the settled game clock and weather; the current crew portrait is composed separately and excludes absent members. The pantry and diner/stage are interiors; the damaged-road delivery stays outdoors.

Each variant has its own title, setup, action label, outcome, and factual source explanation. Narrative is localized in English, Spanish, Italian, and Arabic, with explicit English fallback in the other sixteen supported locales. Existing journal receipts remain records in the language used when the action occurred. Costs and rewards in the receipt come from actual before/after state.

Editorial adaptations preserve the source premise while meeting the cast and staging requirements:

- The diner owner and farmer use feminine or neutral references instead of the original masculine pronouns.
- The self-made billionaire is feminine. Her claim is spoken rather than baked into an unreadable banner. The line about six people lifting it becomes a statement about hired help, so it does not invent six surviving travelers.
- Before frames leave the player’s work unfinished; after frames show gathered produce, sorted groceries, clean dishes, assembled staging, or an unloaded truck. The delivery road stays broken. The empty tills and tip jar stay empty.
- Supporting NPCs are part of the location, not additional crew. No player characters or van are baked into these activity atlases.
- Two food-aid variants use the already verified CRS report as their source instead of the unavailable original article. The explanation describes the law’s projected spending and eligibility changes; the local pantry and dialogue are fiction.

Production files and explicit source-frame bounds are in `review/art-satire/asset-catalog.json`. The importer is `review/art-satire/integrate-activities.py`; the B/C translations and generation prompts are retained alongside it. This slice does not complete the remaining scene families or the overall visual-world goal.
