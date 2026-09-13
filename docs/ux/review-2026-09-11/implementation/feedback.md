# Consolidated player feedback and work order

Updated September 13, 2026. Repeated reports are merged below. Later corrections take precedence: remove the Pants stat, retain the small crew grid, and use compact equal-width controls. This is the requested-behavior ledger; current test results and limitations are recorded in [polish/README.md](polish/README.md).

Production and preview currently serve **d3ed78dac458b64a80ee**. [Final delivery and verification](polish/native-share-focus-d3ed78dac458b64a80ee/README.md). All implementation and delivery items are complete. Physical-phone installation/offline reopening is user-confirmed. Actual native sharing/cancellation and the final Safari keyboard regression have separate recorded evidence. The dated notes below are historical; later corrections supersede them.

## Work completed in order

1. Removed Pants, corrected whole-dollar Cash and unified outcome cards.
2. Corrected local rewards and repeat access; added acknowledged ally departures and journal entries.
3. Applied shared-control, camp, town/store, help, offline-menu, onboarding and ending feedback.
4. Reviewed continuity and fixed starting-stat, camp repair/navigation and first-day route-scale mismatches.
5. Moved all six origins to the West Coast; extended geographic routes, regional scenery and sourced local content.
6. Redesigned Word from the town after the latest visual feedback; verified English, Spanish, Italian and Arabic at desktop and phone widths.
7. Verified the West Coast and conversation pass in installed Chrome and native Safari, refreshed the full inventory and published the local preview at port 8180.
8. Added Receipts to the shared stats bar, wired 26 visible receipt rewards, and connected character/news bonuses. Verified Chrome, offline persistence, outcomes, The Trail and Journal; the native Safari checks also passed.

9. Reconciled the campaign tester with paid starting inventory, actual shelf prices, crew care, explicit repairs and action time.
10. Fixed late-route camp days advancing the van and recorded interactive repair days accurately. Desktop/phone Chrome and native/browser tests pass; the measured 8,000-run campaign still fails the unchanged balance targets.
11. Fixed the map's missing receipt count and made every stats-bar caller supply the real count. Added award, map, town, store and offline persistence coverage.
12. Contained the scrolling Journal so entries cannot overlap the footer, with keyboard access to the full log.
13. Removed the inset green HUD background and aligned encounter subtitles beneath their main text, with numbers in a separate column. Buttons and keyboard shortcuts remain intact.
14. Reverified the full loading gate: all 87 game files are cached before play; failed downloads and missing cached art block launch; retry, offline relaunch and complete-update activation pass current Chrome desktop/phone checks.
15. Excluded Finder metadata from releases after discovering it could fail the strict cache gate on the Safari preview. Corrected that preview server to serve existing extensionless files before applying the app fallback.
16. Completed 18 native Safari feedback checks after the Mac became available.
17. Fixed Help & Tips shifting the layout: hidden tips retain their space and leave the keyboard/accessibility tree. Chrome desktop/phone and native Safari desktop/narrow checks show zero movement. All six native Safari offline/update checks pass on the new build.

Current evidence: [Loading, HUD and journal verification](polish/README.md). The [campaign balance decision](campaign/balance-decision.md) is resolved: retune gameplay and stat management to meet the retained experience tests and automated player expectations. Do not weaken those checks. Prior-run compatibility was explicitly waived by the user; it is not a deliverable.

## Scene art and the crew

- Supply a relevant scene for missing encounters such as Pop-up Mutual Aid.
- Show the entire active crew in a normal-size van, two people per row; do not stretch it into a limousine.
- Compose van occupants dynamically, removing anyone who dies or leaves.
- Rotate the main character in other scenes; avoid repeatedly making the white man in the red cap the protagonist.
- Maintain the established pixel-art style, including tractor trailers; avoid increasing realism.
- Position the character cards consistently over scenes, keeping the portrait and name strip aligned and equal in width.
- Use the player avatar and identity card in the final scene instead of an arbitrary crew member.
- Preserve space for satire as translatable overlays rather than text baked into scene images.

## Start, character selection and crew naming

- Ask for the player's name and crew name during onboarding.
- Generate random gender-neutral first names for other crew members, while allowing unrestricted replacement names.
- Remove the explanatory sentence about having chosen gender-neutral names.
- Use player-facing character language rather than “persona.”
- Remove “Mode · Persona · Outfitting” from the start screen.
- Put Choose your character to the right of the run code, left-aligned with the Deep End option; stack correctly on phones.
- Select a random character on the first visit to character selection.
- Move the journey mission out of the character details panel and give the panel a consistent layout.
- Keep the selected-character/budget summary and Continue button exactly aligned horizontally.
- Restore the smaller crew grid. Show unsquished, appropriately filled bust portraits without backgrounds or frames; do not enlarge the entire grid.
- Include the player's entered name correctly on early termination and final results.

## Outfitting, supplies and money

- Treat starting supplies as a visible, preselected purchase from the starting budget, not hidden base inventory.
- Allow the entire loadout to be customized or emptied, even when that makes the journey harder.
- Fix the next-stop/supply block's bottom spacing.
- Move explanatory supply forecasts into contextual help.
- Offer ways to replenish resources: forage outside town, barter in town, and earn cash through positive encounters or work.
- Keep Receipts in the top stats bar and provide clearly offered ways to earn them. Show actual awards in the outcome, The Trail and Journal, with contextual help explaining their role.
- Use “Cash” on cash outcome cards and whole-dollar amounts for costs, balances and changes.
- Round actual discounted transaction prices to whole dollars so displayed prices and charges match. Round leftover cents in legacy saves up on recovery.

## Shared layout and travel controls

- Compress the primary stats bar. Keep it above the artwork.
- Use one continuous background for the stats bar; remove the inset green fill behind the later stats.
- Only the secondary time/weather/temporary-effects strip covers the top of the graphical scene.
- Put the Your turn/next-stop bar outside and directly below the artwork, with no gap between translucent scene elements.
- Remove the detached help button from the travel controls.
- Keep Camp and Resume travel on the same row. Camp toggles on and off; Resume stays available while camping and leaves camp immediately. Town-camp Resume uses the same departure path as Depart town.
- Replace Normal/Fast/Step choices with a Fast toggle beside Camp; remove Step mode.
- Use compact, consistent control widths rather than stretching them across all available space.
- Keep Resume travel visible during encounters and turns; disable it when necessary.
- Preserve the standard stats, inventory and detail access on all gameplay screens, including town, store and encounters.
- Avoid page jumps, flicker and out-of-view feedback after lower-page actions.
- Keep leg progress and the scrolling road smooth during automatic travel.
- Remove empty narration such as “The next encounter is ready. You made progress along the route. Encounter!” and “On the road. Travel continues until a decision, arrival or danger needs you.”
- Remove the repeated When and where block where the same information already appears elsewhere.

## The Trail, Conditions, The Van, Journal and help

- Put the recent-action information in the same tabbed area as Conditions, The Van and Journal.
- Rename Trail report to The Trail.
- Redesign Last Action to fit the game rather than appear as a stray text component.
- Remove stale encounter-result cards from town screens; preserve those results in the journal.
- Redesign journal entries with readable dates, locations, narrative and resource changes.
- Keep the journal's scrollable content contained above the footer and usable from the keyboard.
- Fix clipped/mispositioned tooltips.
- Add a persistent Help & tips setting, on by default. Off hides contextual-help buttons but leaves informational buttons available.
- Explain how mutable values such as allies and morale increase and decrease.
- Remove the redundant low-sanity modal; retain the red stat and its initial three-second pulse.

## Weather and temporary effects

- Apply weather changes and costs without a confirmation or forced camp interaction.
- Show actual impacts and a brief change notification near the weather indicator at the top of the scene.
- Present temporary policy effects such as Travel Ban Lite in the same area, with their meaning, duration and player impact.
- Use concise “Breakdown risk +10%” wording.

## Route and map

- Use a geographically accurate U.S. map with the route traced and upcoming towns and services identifiable.
- Give every character a West Coast starting point: Seattle, Portland, San Francisco, Los Angeles, Sacramento or San Diego.
- Align route position, regional scene art and encounter geographic references.
- Always call the destination D.C., not Washington.
- Present the map periodically as a scene, not a permanent additional component.
- Always frame the route being traveled in the zoomed-in view.
- Use the map as another opportunity for regional satire while preserving geographic accuracy.
- Automatically displayed maps dismiss after three seconds unless held; Fast mode never opens them automatically. Manually opened maps show Close map and return to the previous screen without advancing travel.
- Keep manually opened maps open until the player chooses to leave them.
- Make Resume on the map fully resume travel in one click, including after leaving town.
- Do not let a new encounter immediately replace an automatically displayed map; queue the decision until the map closes.
- Place the automatic-resume countdown directly under the map's action buttons.
- Put required U.S. Census/OpenStreetMap attribution at the bottom right of the header, aligned with the current-location line above the map.

## Towns, trading and local conversations

- Replace unclear store/trade glyphs with recognizable service icons and invokable explanations; replace the rejected barter icon with a clearer design.
- Label the shop action Visit the Store.
- Open a trading screen with multiple exchanges rather than putting a single trade on the town button.
- Stack Talk to locals, Trade with locals and Visit the Store on the right inside the route-stop box, in that order.
- Put costs and benefits in smaller, quieter text below each button's primary label.
- Keep Talk to locals available after its benefit is claimed. Strike out the actual benefit while preserving access to the conversation.
- Fix the reversed-looking credibility display: show the available reward initially and the actual claimed reward afterward, including zero at the cap.
- Use the route-stop number/name area for useful town facts, vital statistics and notable attractions. Show the latest official Census population estimates, with their year and a matching source link.
- Redesign Word from the town as a readable local conversation, with a prominent remark, a compact sourced record, aligned spacing and a clear return action.
- Reserve sharper satire for Talk to locals: interesting, sourced facts about Trump-administration impacts in the actual location, with a related fictional joke.
- Show the town scene, the player avatar and a random NPC for local conversations.
- Center “You have completed this stop’s work offer…” under both community-offer buttons with proper spacing.
- Align Leave store vertically with Outfit your van and horizontally to the right beneath Resume. Redesign shopping as compact item rows, direct quantities, carried-part counts and one checkout area; town purchases return directly to town.
- Preserve established spacing and alignment throughout the route-stop panel and actions.

## Encounters, repairs, camp and allies

- Use Oregon Trail as the flow/feel target, slightly modernized; review consistency and continuity and implement the findings.
- Expand the encounter bank substantially, reduce repetition and add regional encounters and positive cash opportunities.
- Keep satire recognizably connected to Trump-administration missteps or faux pas. Sync approved copy from the supplied six-tab Google Doc; preserve costs, rewards and political context and do not ship editorial placeholders.
- Maintain a complete encounter/options inventory and the equivalent for town content to expose missed opportunities.
- Give crew-care situations an explicit satirical cause, such as raw-milk poisoning.
- Identify actual failed parts, such as battery or alternator, and show the cost of every available repair choice.
- Provide barter/local-communication alternatives when no spare, shop stock or cash is available.
- Put guided foraging and farm gleaning on the camp screen outside towns, with cooldown presentation matching other camp actions.
- Match the camp Back to the road button to the encounter version.
- Replace The Consequences with a single Outcome heading in the same small uppercase style.
- Show all actual resource changes with the same stat-card presentation, including cash, vehicle percentage, spare parts and evidence.
- Align each encounter button's subtitle with its primary text, placing the number in its own column; keep these as buttons rather than a numbered list.
- Remove Pants Danger/Meter entirely from rules, choices, weather, UI, help and scoring. This supersedes the earlier request to make it more volatile.
- Give ally losses a dedicated screen with a clear change and a reason, including varied political-satire messages; keep the result in the journal.

## Journey outcomes

- Provide a way to abandon the trail.
- Make ending text and imagery reflect where the player actually stopped and why; do not claim an early loss happened on the steps in D.C.
- Remove the scorecard expander and display the complete scorecard clearly.
- Show the player's name, character avatar and info card in the final scene.
- Keep action confirmations visible without forcing a scroll to the top or changing header height.

- Toggling Help & Tips must not move or resize the stats, scene, controls or menu; optional help stays hidden and information buttons remain available.

## Offline play, installation, preview and review access

- Download and cache every asset behind the loading screen before gameplay. Do not launch with an incomplete cache, even when online; failure must offer retry. Repeated explicitly in the latest feedback.
- Support full offline play after a successful first visit.
- Support home-screen/app-style installation on compatible devices.
- When connected and a newer build exists, finish updating before starting play; retain a complete cached version if an update fails.
- Fix the offline-status/help/install presentation. Keep Ready for offline play centered and last in the menu.
- Verify in actual Chrome and Safari, not Chrome alone. Native Safari checks resumed after the Mac was unlocked and Safari left idle.
- Keep the preview on the requested port 8180 and provide a working URL while changes are in progress.
- Make inventory locations easy to find: [browsable inventory](content/index.html), [choices CSV](content/encounter-options.csv), [complete JSON](content/inventory.json), and [town conversation sources](scene-flow/town-conversations.md).

## Active design limits

- The current bank has 65 encounters, 187 choices and 51 town records. Eighteen runtime encounter compositions include six new interior scenes; the inventory identifies shared art and every encounter mapping.
- The earlier native Safari 26.6.2 unit passed its feedback and loading/update/help-layout checks; final native Safari validation of the latest release remains pending. It rendered the conversation at a measured 600 px width. Phone-sized Chrome layouts pass. Physical-device home-screen installation has not been performed.
- Source citations refer to dated records. Fictional dialogue and ally messages are game satire, not quotations from real residents or reports about named real people.

## September 12 additions

The six-tab workshop was compared against 59 encounter entries, 28 town comments and 14 crew/ally messages. Four encounter entries changed; the rest already match, sometimes with an existing policy-context sentence around the supplied lead-in. The first of each alternative opening was selected. Three unfinished response placeholders were omitted. Full source revision, changed fields and translations are in [content/workshop-sync.json](content/workshop-sync.json).

All 51 route populations now come from the Census Bureau’s Vintage 2025 city estimates (July 1, 2025), published May 2026. The exact selected rows, Census links and dataset hash are in [content/census-populations.json](content/census-populations.json). The game displays the data year and caches the values for offline play.


## Latest interaction and layout corrections

- Show every current-turn journal entry under The Trail, newest first; keep the full history in Journal.
- Move Van crew above parts/equipment; give the trip-summary cards images and a section heading.
- Add a blurred black outline to scene text over images.
- Keep the outcome return action reachable without scrolling through the full report.
- Put Journal date/time, pace and information diet beneath the route, with icons, and retain the settings from the original action.
- Use the same compact stat-card size in Trail and Journal, placing changes on the right when space allows and wrapping on narrow screens.
- Manual Route views say Close map, return to the previous screen, and never start travel. Automatic maps retain their automatic continuation behavior.
- Clicking an active Trail, Conditions, Van or Journal tab collapses its content and removes the active state; clicking again reopens it. Keep keyboard access and saved selection.

These are delivered in preview 56213723b1c62ed89b93. The outcome unit was published first at 9e96345a048fc846c1f3, and subsequent compact-entry/navigation changes were published after interaction and offline checks. The [current verification record](polish/README.md) retains the corrected phone clipping and the unresolved Chrome shutdown timeouts separately from gameplay assertions. The broader campaign acceptance run remains open; the user has chosen retuning while retaining the checks.


## Latest feedback — mandatory titles, tab finish, and campaign retuning

- Every Trail entry must show a title, using Traveled for routine travel.
- Remove the Journal container top divider and round the active tab corners.
- Retune gameplay to meet the existing campaign tests. The tests define the expected experience; fix stats management without relaxing acceptance thresholds.

- **Modern road travel and balance:** Steady roughly 100–200 miles/day, Heated 200–300, Blitz 300+. Retune condition and encounter probabilities and resource costs together against the pinned experience tests and existing automated player expectations. Only superseded mileage and trip-duration targets change; other acceptance checks remain authoritative.

- **Hourly pacing correction:** Replace fixed miles/day with a travel-rate cap: Steady up to 60 MPH, higher caps for Heated and Blitz. Compute miles from actual driving hours. Every hour spent gathering, repairing, receiving care or doing town activities must reduce available driving time. This supersedes the interim 150/250/350 fixed daily targets; probability and resource retuning remains required.

## September 12 additional feedback

- Remove top-corner flecks from rounded active tabs: fixed and published in `0f97e366c7a71602378d`.
- Paid-shift completion must clearly return to town. Navigation outcomes use a return arrow, and the driving icon is reserved for starting travel. Candidate implemented; functional checks pending.
- Move all lower status-strip information except turn text into the lower top HUD, delete the bottom strip and turn text, and show the present crew standing outside the stopped van. Candidate implemented; art and layout checks pending.
- Update the live preview after each verified item; do not batch every remaining change into a final-only update.
- Recheck all appropriate V3 source tabs: complete, no revision or runtime copy mismatch. Evidence in `content/v3-recheck-2026-09-12.json`.

Completed newest HUD, stopped-crew, and return-navigation feedback in preview `ae87262a0b1ff468dbf4` after desktop/phone, native Safari, offline and Wasm checks. The hourly balance pass remains open.

- Clarification completed in preview `08a07362f3e5c5445818`: when stopped on the road, park the van on the lower shoulder and group all present crew to its right. Desktop, phone, native Safari and offline verification passed.
- Investigating the reported ~20 miles/day in the preview. The older engine produced about 19.2 physical miles/day on the Staffer route; running Wasm remains pinned until reload. Fresh-browser clock/distance verification is in progress, with parallel mileage, browser and balance audits.

## Latest scene, release, homepage and Journal feedback

- Crew must gather to discuss the player's decisions, with readable faces and staggered depth beside the parked van. Both van and crew need proper foreground occlusion. Delivered locally in `135051c37b00eb6eeff0`; 14 Chrome desktop/phone checks, the unmodified game client and the native Safari crew/offline check passed. Safari's broader run timed out later and remains incomplete.
- Explicitly authorized a deployment agent to publish the most recently available preview to production. Completed for `08a07362f3e5c5445818` at https://dystrail.com/play/; all 168 published files and an updated installation's offline restart were verified. Newer local scene/balance changes were excluded.
- Requested a phone push on deployment. The app completion alert is configured; phone delivery is not verifiable from the available tools.
- Replace the homepage's obsolete no-download sentence with “…even offline!” on a smaller centered warm-brown panel emerging behind Play in browser. Phone screenshots were shown and the user explicitly approved the exact treatment. Publication includes a versioned stylesheet URL to avoid stale styling.
- Journal should have one entry per day, accumulating today's actions and outcomes rather than showing nearly identical hourly cards. Keep meaningful detail, daily resource totals and the current-turn Trail feed. This is implemented and verified in the current release.


## Mobile status bars and impact explanations — September 12

The latest phone screenshots show resource separators breaking across rows, weather/policy effects stacked in a narrow right column, and a dense Book Panic explanation. The requested follow-up is a coherent mobile status layout and clean formatting throughout impact help. The current implementation uses four-plus-three resource groups on phones, full-width settings/trip/effect rows, structured policy costs/protection/timing, singular/plural duration, and a shared viewport-contained popup with an explicit close action. This intermediate layout was superseded by the September 13 compact HUD: icon/value readouts, weather and policy names with info controls, and all cost/duration details inside those controls. Production and preview serve a9ff83cd924a07fdf4ec; the original phone issue remains here as historical feedback.

The approved homepage CTA is now live. Final Pages run 34732895218 publishes only index.html and styles.css; all 168 public files match the release artifact, with the other 166 files unchanged. Phone screenshot: /tmp/dystrail-homepage-release-prep/production-homepage-phone.png.
