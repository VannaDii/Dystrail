# Community exchange scenes

All nine workshop variants for spare tires, batteries, and supplies now have separate offered and completed pixel-art frames. Each pair keeps the same resident, table, and goods. Goods move from the resident’s side to the player’s side, or in the opposite direction for payment. The vehicle is never shown being repaired: these exchanges pack a spare or exchange one for food.

The resident is part of the location, not an extra traveler. The current crew portrait is composed separately from the surviving party. No player characters, lettering, prices, or interface elements are baked into the three atlases. The workshop’s fixed references to two travelers are represented by the current crew rather than invented people. Portraits, weather, and time come from the real state.

Each offer uses a presentation-only selection identified by family, route, and stop. The selection is sealed when the exchange screen opens. Browsing, reloading, and importing saves do not draw simulation randomness. A successful transaction freezes that identity in the recovery report, so the completed picture and prose remain tied to the chosen offer.

The existing engine still owns the exchange: three supplies buy a spare tire, four buy a spare battery, and one spare tire buys five supplies. There is one exchange per stop across all three offers. The existing thirty-minute clock cost is shown before the action. Missing supplies, a missing tire, and insufficient carrying room have visible explanations. Exact costs remain appropriate for these explicit trades.

Only a successful atomic exchange opens the completed scene. A shared action lock prevents rapid repeated clicks. The outcome uses actual inventory receipts, and its exit follows the resulting game state so a newly required decision takes precedence over returning to town. Manual saves retain the goods and used-stop restriction; recovery checkpoints also retain the illustrated outcome.

English, Spanish, Italian, and Arabic have complete per-variant titles, setup, actions, and outcomes. The other sixteen locale bundles have explicit English fallback. Source help follows the selected variant. The food-assistance A variant uses the already verified CRS report shared with pantry work, replacing the unavailable workshop AP address. The other hooks reuse verified energy-order, tariff-incidence, campaign-finance, and Medicaid-law sources with their qualifications.

Production provenance, exact atlas bounds, and source-record mapping are in `review/art-satire/asset-catalog.json`. `review/art-satire/integrate-barter.py` imports the copy and source bindings; it marks records pending until rendered review is recorded. Validation evidence is in `review/art-satire/barter-review.json`.

Rest remains a separate pending family. Its three variants need composition around the actual parked van and remaining crew, not the exchange residents. Rest artwork must not imply guaranteed net recovery: the existing daily costs and weather still settle. Rest before the hearing must return to hearing preparation and preserve its uncommitted state. Those boundaries are retained for the next scene slice.
