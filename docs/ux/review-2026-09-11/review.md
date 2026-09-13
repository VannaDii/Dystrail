# Dystopian Trail — consistency, continuity, flow and feel

September 11, 2026 · Current local release preview · Review only

The art promises a journey. The interaction still makes the player repeatedly authorize the next screen. The most valuable next pass is the travel loop and its continuity rules: sustained movement, legible interruptions, consequences that remain true, and arrivals that feel like places.

The classic 1985 Oregon Trail is the primary experience reference. Its lead designer describes automatic travel between landmarks, interrupted by important events or a player pause, with visible conditions that invite intervention. That is the central behavior to recover. [R. Philip Bouchard, The Travel Screen](https://www.philipbouchard.com/oregon-trail/travel-screen.html)

Keep our established direction: Dystopian Trail in the UX; D.C. as the destination; real U.S. geography and persona-specific origins; occasional map scenes; the normal six-person van; generous contextual help; and modern institutional satire. Modernization should make decisions understandable and recovery reliable while giving travel space to unfold.

## Evidence and limits

I used the current release files on an isolated local origin, port 8082, so the user's port 8081 save was not changed. The review includes a natural Classic / Journalist opening through Day 3 using run code CL-ORANGE42, then controlled imports for La Crosse, absent crew, failure, and the final vote. The test player was Alex and the crew was The Paper Tigers. Desktop captures are 1440 × 900; phone captures are 390 × 844. Source inspection supports causes and unexercised mechanics. Parent commit: e38e5ab, with substantial existing uncommitted changes; the screenshots describe the served build, not a clean commit.

This is a heuristic review and directed playthrough, not a player study, a full route playthrough, a balance certification, a complete accessibility audit, or verification of all languages and personas. Fixture results establish rendering and interaction, not how often those states occur naturally. No game source, assets or rules were changed. Rust and release pipelines were not rerun for this review-only artifact.

Screenshots are fresh and unretouched. Scroll positions that hide the scene are evidence, not cropped mockups. The earlier full-page capture artifact was excluded; setup and persona were recaptured at viewport size. The duplicate 07 capture is omitted. The interactive report uses existing screens as references, clearly separate from proposed behavior.

## What the comparison means for this game

| Dimension | Oregon Trail reference | Current Dystopian Trail | Direction |
|---|---|---|---|
| Travel rhythm | Automatic travel between meaningful interruptions | A fixed animation between player-authorized engine actions | Sustained travel with pause, speed and accessible step controls |
| Landmarks | Places break the journey into anticipated legs | Accurate map; town services below ordinary road controls | Approach cue, local arrival scene and a deliberate departure |
| Party | Named people create personal stakes | Named roster and dynamic van; mostly global consequences | Named incidents, persistent status and active scene casting |
| Preparation | Equipment choices connect to survival | Catalog costs and stat grants, little next-leg context | Explain useful stock, protection and what the next leg needs |
| Cause and effect | Visible conditions invite intervention | Known-effect labels disagree with caps; receipts can be stale | Honest forecasts plus one persistent action history |
| Geography | Terrain and climate support a real route | Real U.S. geometry and route captions now work together | Keep route authority; deepen arrivals and local encounter context |
| Emotional rhythm | Routine, trouble, relief and attachment | Frequent similar panels with a constant sardonic voice | Quiet road time, pointed institutional satire and sincere mutual aid |

Named party members were central to the 1985 game's emotional investment: illness and death identified the affected person. Shops and landmarks also connected advice to people. Our named crew should fulfill that same emotional promise. [Bouchard, Including Humans](https://www.philipbouchard.com/oregon-trail/including-humans.html)

The original geographic design made terrain and place structurally important. Our accurate map is the right foundation; arrival scenes and local encounters should now carry that geography into play. [Bouchard, Real Geography](https://www.philipbouchard.com/oregon-trail/real-geography.html)

For a modest modern extension, Gameloft's version connects character traits to outcomes and ties equipment to survival and wagon condition. Borrow that legibility and character relevance. A larger inventory puzzle, collectible progression system, or extra minigame is not needed to solve the gaps found here. [Gameloft: Party Drafting](https://www.gameloft.com/article/play-tips-1-party-drafting), [Shopping](https://www.gameloft.com/article/play-tips-2-shopping), [Inventory](https://www.gameloft.com/article/play-tips-3-inventory)

## Trace of the sampled experience

1. **Setup:** choose Classic or The Deep End; run code is prominent. The progress text says Mode · Persona · Outfitting, omitting Crew. The opening establishes survival and D.C., but not the final political objective.
2. **Persona:** Journalist selects Minneapolis. The origin line is effectively invisible against the light preview. Receipts and the score multiplier are unexplained. The six portraits are distinctive.
3. **Crew:** editable names and gender-neutral companion defaults work. The player name is last in this persona's six-field order. Names establish an expectation of personal consequences.
4. **Outfitting and review:** bought a ration pack, a tire and warm coats for $30. Cash and cart arithmetic are legible; capacity and useful preparation are less so. The ration gain was capped from +3 to +2.
5. **Departure:** landed at the bottom of the travel page. The road scene was above the viewport. This undermines the art's ability to announce the new stage.
6. **Travel:** one command played a fixed animation, then a map after approximately 3 physical miles. Continuing revealed Heat Wave and Civic Center Potluck. Geographic captions were coherent; interruption priority was not.
7. **Choice:** Stay for stew offered +3 supplies and -2 Pants danger, but caps meant +0 and -1. The outcome showed the realized changes without explaining the difference.
8. **Back to road:** Last action still described the preceding travel rather than the stew. Continue today completed the remaining day; Day 2 offered Travel one day again.
9. **Camp:** free entry advanced the displayed hour. Rest spent one day and one supply, restored one sanity at the cap, and invisibly advanced mileage. Returning to the road moved the clock backward. The visible journal omitted rest.
10. **Controlled town arrival:** La Crosse had working trade and store controls, below the ordinary road configuration and details. Trading updated resources correctly. The store still described initial persona gear.
11. **Controlled absent crew:** the van removed two members correctly; the Organizer's likeness remained in the potluck art after that member had departed.
12. **Controlled ending and recovery:** the final vote resolved and the terminal screen returned after reload. Endings emphasized score and system fields; names and crew fates were absent. This does not establish end-to-end balance or natural success probability.

## Findings in recommended order

P1 means address in the next continuity pass because it obstructs the intended experience or trust in state. P2 means follow in the scene, language and narrative pass. These are design priorities, not claims that every item blocks execution.

### F01 · P1 · Let the trip continue between meaningful interruptions

**Evidence:** Experience gap. Travel resolves one engine action, plays a fixed 2.6-second animation, then stops. An encounter can require a map confirmation, a choice, an outcome confirmation and another travel command. The first natural encounter occurred only 3 physical miles into Day 1.

**Why it matters:** Suspense becomes a predictable wait for another form. The player cannot settle into watching the van, conditions and approaching town.

**Change:** Make Depart / Resume start sustained travel. Pause for a consequential event, arrival, critical condition or deliberate player stop. Provide pause and readable travel speeds; an optional step mode can remain. Reveal decisions as interruptions inside the journey.

**Acceptance:** One Resume carries the crew through uneventful days to the next meaningful interruption. The player can pause at any time; no hidden-tab travel or skipped decisions. Reload preserves the pending decision exactly once.

**Owner:** Travel orchestration and interaction design.

**Screens:** [08-transit](screenshots/08-transit.png), [09-map-interlude](screenshots/09-map-interlude.png), [10-first-encounter](screenshots/10-first-encounter.png), [11-encounter-result](screenshots/11-encounter-result.png).

**Source:** [dystrail-web/src/app/turn.rs:12](/Users/vanna/Source/Dystrail/dystrail-web/src/app/turn.rs:12), [dystrail-web/src/app/view/handlers/travel.rs:8](/Users/vanna/Source/Dystrail/dystrail-web/src/app/view/handlers/travel.rs:8).

### F02 · P1 · A new stage can open below its scene and primary action

**Evidence:** Observed bug. Starting the journey from the checkout review landed at scrollTop 930 in an 1830-pixel document on a 900-pixel viewport. Inventory and Camp were visible; the road and HUD were above the viewport. Opening camp and a town store also inherited the prior scroll position.

**Why it matters:** The player misses the visual transition and must search for the next action. This also makes scene changes appear absent even when the asset is correct.

**Change:** Define entry positions per stage and return positions for detail views. Keep scene, essential state and next action in a coherent viewport. Restore useful scroll deliberately without forcing mouse users onto a visibly focused heading.

**Acceptance:** Depart, encounter, outcome, camp, store and map transitions expose the new context and next action at desktop and phone sizes. Keyboard focus remains visible when keyboard navigation is used.

**Owner:** App navigation and scene layout.

**Screens:** [05-departure-review](screenshots/05-departure-review.png), [06-travel-idle](screenshots/06-travel-idle.png), [12-camp-arrival](screenshots/12-camp-arrival.png), [18-town-store](screenshots/18-town-store.png).

**Source:** [dystrail-web/src/app/view/phases/aftermath.rs:10](/Users/vanna/Source/Dystrail/dystrail-web/src/app/view/phases/aftermath.rs:10), [dystrail-web/static/journey.css:3](/Users/vanna/Source/Dystrail/dystrail-web/static/journey.css:3).

### F03 · P1 · The latest consequence is not the latest story

**Evidence:** Observed bug. After the potluck restored Sanity +1 and reduced Pants danger by 1, the road still showed the earlier travel receipt with Sanity -2 and Pants danger +1. After resting, Last action still said Traveled, and the visible journal omitted the rest. Import replaced the visible history with Save loaded.

**Why it matters:** The player cannot reliably connect a choice to the current state or reconstruct what happened after returning to a save.

**Change:** Use one ordered action history for travel, choices, rest, repairs and trades. Derive the latest receipt and journal from it. Include day, place, named participants, actual deltas and why they occurred; preserve history through manual load as well as autosave.

**Acceptance:** The most recent action, HUD deltas and first journal entry agree before and after reload / save import. A rest entry and a trade entry survive recovery.

**Owner:** Action receipts, persistence and journal.

**Screens:** [11-encounter-result](screenshots/11-encounter-result.png), [13-camp-rest](screenshots/13-camp-rest.png), [14-journal](screenshots/14-journal.png), [17-town-services](screenshots/17-town-services.png).

**Source:** [dystrail-web/src/app/view/handlers/travel.rs:135](/Users/vanna/Source/Dystrail/dystrail-web/src/app/view/handlers/travel.rs:135), [dystrail-web/src/app/view/phases/camp.rs:36](/Users/vanna/Source/Dystrail/dystrail-web/src/app/view/phases/camp.rs:36), [dystrail-web/src/app/view/handlers/storage.rs:28](/Users/vanna/Source/Dystrail/dystrail-web/src/app/view/handlers/storage.rs:28).

### F04 · P1 · Known effects need to mean what this crew will actually receive

**Evidence:** Observed mismatch. The Journalist started with 18 supplies. A $5 ration pack advertised +3 but checkout capped the total at 20, charging full price for only +2. At 20 supplies, Stay for stew advertised Supplies +3 but delivered zero; Pants danger -2 became -1 at the floor. Rest advertised Sanity +4 but delivered +1 at the cap.

**Why it matters:** The arithmetic is internally clamped, but the interface teaches an unreliable decision contract. Outfitting cannot answer how long the crew can travel or what the first leg needs.

**Change:** Preview effective gains, costs and caps using the actual state. Explain overflow before purchase or choice. Present current stock, useful capacity, projected consumption and distance to the next supply stop. Describe protection as a player benefit, replacing equipment-tag language.

**Acceptance:** Every deterministic preview matches the realized delta, or explicitly distinguishes base effect from a cap, protection or uncertainty. A purchase cannot silently waste supplies.

**Owner:** Outfitting, encounter previews and camp previews.

**Screens:** [04-outfitting](screenshots/04-outfitting.png), [05-departure-review](screenshots/05-departure-review.png), [10-first-encounter](screenshots/10-first-encounter.png), [11-encounter-result](screenshots/11-encounter-result.png), [13-camp-rest](screenshots/13-camp-rest.png).

**Source:** [dystrail-web/src/components/ui/encounter_card.rs:10](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/encounter_card.rs:10), [dystrail-web/src/components/ui/camp_panel/mod.rs:76](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/camp_panel/mod.rs:76), [dystrail-web/src/components/ui/outfitting_store/planner.rs:1](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/outfitting_store/planner.rs:1).

### F05 · P1 · Time, rest and weather need one believable account

**Evidence:** Observed + source confirmed. Opening camp changed Day 2 at 08:00 to Day 2 at 20:00 without spending time. Rest produced Day 3 at 20:00; returning to the road reset it to Day 3 at 08:00. Rest also advanced route mileage from 7 to 12 physical miles through a hidden travel-credit rule. Heat Wave reports Supplies +1, which the engine really adds; Check conditions only dismisses the warning.

**Why it matters:** The clock appears authoritative while behaving like decoration. A stationary recovery day advances geography, and weather mixes harmful language with a supply bonus that has no explanation.

**Change:** Drive calendar, lighting and route advancement from persisted simulation time. Define a stationary rest day or explicitly offer a short-drive-and-rest action. Review the weather supply sign and explain the chosen mechanic. Make Check conditions actually show the actionable condition.

**Acceptance:** Free navigation never changes time. The displayed clock cannot run backward. Rest, travel distance, weather costs and location tell the same story; danger information appears before a routine map interlude.

**Owner:** Day accounting, world clock and weather feedback.

**Screens:** [08-transit](screenshots/08-transit.png), [12-camp-arrival](screenshots/12-camp-arrival.png), [13-camp-rest](screenshots/13-camp-rest.png), [14-journal](screenshots/14-journal.png).

**Source:** [dystrail-web/src/components/ui/world_view.rs:40](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/world_view.rs:40), [dystrail-web/src/components/ui/game_clock.rs:25](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/game_clock.rs:25), [dystrail-game/src/camp.rs:118](/Users/vanna/Source/Dystrail/dystrail-game/src/camp.rs:118), [dystrail-game/src/weather.rs:407](/Users/vanna/Source/Dystrail/dystrail-game/src/weather.rs:407), [dystrail-web/src/app/view/mod.rs:58](/Users/vanna/Source/Dystrail/dystrail-web/src/app/view/mod.rs:58).

### F06 · P1 · Arriving in town should change the scene and the available plan

**Evidence:** Observed experience gap. At La Crosse, the page still led with The road to D.C. and Travel one day. Town services began at y=1287 on a 900-pixel viewport with the journal selected. The exchange worked and consumed resources, but offered the same 3 supplies for 1 tire pattern. The store reused departure copy about persona starting gear.

**Why it matters:** A place worth reaching becomes an easy-to-miss footer. The player can leave without noticing resupply, and towns lack distinct social or strategic value.

**Change:** Give arrivals a local establishing scene, town name, next-leg context and Shop / Trade / Talk / Depart actions. Let towns differ through stocked goods, needs, prices or information. Keep departure explicit so services cannot disappear unnoticed.

**Acceptance:** Arrival visibly interrupts travel. The local services and next stop are clear without scrolling through pace, diet and inventory. Transactions retain the town context and show costs immediately.

**Owner:** Route services, arrival UI and narrative content.

**Screens:** [16-town-arrival-top](screenshots/16-town-arrival-top.png), [17-town-services](screenshots/17-town-services.png), [18-town-store](screenshots/18-town-store.png).

**Source:** [dystrail-web/src/app/services.rs:7](/Users/vanna/Source/Dystrail/dystrail-web/src/app/services.rs:7), [dystrail-game/src/route_services.rs:1](/Users/vanna/Source/Dystrail/dystrail-game/src/route_services.rs:1).

### F07 · P1 · The named crew must remain the same people throughout the journey

**Evidence:** Controlled-state continuity failure. The van correctly removed Charlie (dead) and Robin (departed) from an imported test roster. The next potluck illustration still featured Robin's Organizer likeness. Current member records carry names, personas and status; no gameplay callers of the member-status mutation were found. The result identifies Journalist, not Alex or The Paper Tigers.

**Why it matters:** Naming creates an emotional promise that generic text, static encounter casts and score-first endings do not yet keep. A removed character can appear to return.

**Change:** Make named crew affect incidents, aid, illness, recovery and departure. Use the active roster when choosing scene subjects; record which crew member participated. Preserve identity in camp, outcomes and endings. Maintain distinct visual identities for crew and opposing officials.

**Acceptance:** A natural encounter can affect a named member and leave a persistent consequence. Absent members are excluded from all active-party imagery. Each ending explains what happened to the named crew.

**Owner:** Party mechanics, scene casting and narrative.

**Screens:** [03-crew](screenshots/03-crew.png), [19-absent-crew-van](screenshots/19-absent-crew-van.png), [20-absent-crew-encounter](screenshots/20-absent-crew-encounter.png), [24-failure-ending](screenshots/24-failure-ending.png).

**Source:** [dystrail-game/src/party.rs:1](/Users/vanna/Source/Dystrail/dystrail-game/src/party.rs:1), [dystrail-web/src/components/ui/journey_scene/mod.rs:50](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/journey_scene/mod.rs:50), [dystrail-web/src/components/ui/journey_scene/encounters.rs:1](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/journey_scene/encounters.rs:1).

### F08 · P1 · The persona's origin is almost invisible

**Evidence:** Measured readability bug. Starts in Minneapolis · Bound for D.C. uses #f6d68b on #eadcb8 in the persona preview. Computed-color contrast is approximately 1.03:1 at 17px, regular weight.

**Why it matters:** The newly important starting-location choice is visually lost at the moment the player selects it.

**Change:** Use the dark text token on this cream surface and apply a surface-aware text rule to every preview and help panel. Explain the score multiplier separately from the persona's practical benefit.

**Acceptance:** Origin and destination read clearly in the default theme, high contrast, and supported text sizes. Normal text meets the project's 4.5:1 requirement.

**Owner:** Persona preview and design tokens.

**Screens:** [02-persona](screenshots/02-persona.png).

**Source:** [dystrail-web/static/journey.css:72](/Users/vanna/Source/Dystrail/dystrail-web/static/journey.css:72), [dystrail-web/src/components/ui/persona_select/preview.rs:1](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/persona_select/preview.rs:1).

### F09 · P2 · The HUD shows numbers before it explains the survival model

**Evidence:** Information hierarchy gap. Seven stats have equal weight. Cash and vehicle condition sit in inventory, distance to the next town appears only on the map, and Supplies has no duration or unit. Receipt chance, Allies and the crew roster describe different systems without a clear relationship. Fuel / Food contains food and water but no fuel purchase.

**Why it matters:** The player sees many numbers but cannot quickly answer: can we make the next stop, who is struggling, and what needs attention now?

**Change:** Group physical survival, emotional strain and public influence while keeping all requested stats available. Keep vehicle condition, supplies outlook and next-town distance near travel. Define Supplies, Receipts and Allies consistently. Show pace and information-diet comparisons together when assessing the situation, without click-to-read accordions.

**Acceptance:** A first-time player can explain the next leg's main constraint and each major stat in their own words. Critical information stays visible during travel at phone and desktop sizes.

**Owner:** HUD, terminology and pause view.

**Screens:** [15-top-menu](screenshots/15-top-menu.png), [14-journal](screenshots/14-journal.png), [21-mobile-road](screenshots/21-mobile-road.png), [04-outfitting](screenshots/04-outfitting.png).

**Source:** [dystrail-web/src/components/ui/stats_bar/mod.rs:1](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/stats_bar/mod.rs:1), [dystrail-web/src/components/ui/travel_panel/status.rs:1](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/travel_panel/status.rs:1).

### F10 · P2 · Keep the map as a scene, but let urgency decide when it appears

**Evidence:** Interruption ordering gap. The current map is an actual U.S. map with a traced road route, upcoming towns, route zoom and regional satire. It appears on first progress, then at checkpoint changes including five-day intervals. In the natural run it preceded a waiting Heat Wave notice and potluck encounter. Its full overview also puts continuation below the first desktop viewport.

**Why it matters:** A useful orientation beat competes with unresolved consequences. The first map stop comes before the player has experienced a meaningful stretch of travel.

**Change:** Retain the full-scene map and accurate geometry. Show it at safe leg transitions, periodic travel checkpoints and deliberate route review; queue it behind urgent incidents. Keep route, upcoming services and continuation together. Tie marginal satire to the actual region without moving places or obstructing route labels.

**Acceptance:** The map never buries an unresolved critical event. Every scene caption, map position and town service refers to the same place. It remains an occasional scene, never a permanent extra panel.

**Owner:** Map scheduling and scene composition.

**Screens:** [09-map-interlude](screenshots/09-map-interlude.png), [29-map-overview](screenshots/29-map-overview.png), [30-map-route](screenshots/30-map-route.png).

**Source:** [dystrail-web/src/app/map.rs:7](/Users/vanna/Source/Dystrail/dystrail-web/src/app/map.rs:7), [dystrail-game/src/route.rs:142](/Users/vanna/Source/Dystrail/dystrail-game/src/route.rs:142), [dystrail-web/src/components/ui/route_map/mod.rs:26](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/route_map/mod.rs:26).

### F11 · P2 · The ending needs to pay off the journey the opening promised

**Evidence:** Narrative and goal gap. The opening asks the player to reach D.C. intact. A controlled final state introduced Filibuster Boss, a one-action vote with round costs but no clear explanation of how the journey's receipts, credibility or allies affect the vote. Endings show twelve statistical fields, including Score Threshold and Boss Vote Ready, but no named crew recap.

**Why it matters:** Arrival changes the apparent objective late, and a personal road trip ends as a system scorecard. Generic Live with it. text also gives beneficial encounters and recovery the same emotional cadence as disasters.

**Change:** Establish the purpose of reaching D.C. during onboarding and build anticipation across route milestones. At the final challenge, show which accumulated strengths and harms matter. End with the crew, route and decisive moments, then optional detailed scoring. Let relief, loss and absurdity have different aftermath tones.

**Acceptance:** Before departing, players can explain why they are going to D.C. At the end they can explain the outcome through their own decisions, and see the fate of their crew.

**Owner:** Onboarding, final challenge and results.

**Screens:** [01-setup](screenshots/01-setup.png), [27-final-vote](screenshots/27-final-vote.png), [24-failure-ending](screenshots/24-failure-ending.png), [25-ending-scorecard](screenshots/25-ending-scorecard.png).

**Source:** [dystrail-web/src/app/view/phases/boss.rs:1](/Users/vanna/Source/Dystrail/dystrail-web/src/app/view/phases/boss.rs:1), [dystrail-web/src/components/ui/result_screen/layout.rs:1](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/result_screen/layout.rs:1), [dystrail-web/src/app/view/phases/aftermath.rs:28](/Users/vanna/Source/Dystrail/dystrail-web/src/app/view/phases/aftermath.rs:28).

### F12 · P2 · Give each scene a specific job and a consistent cast

**Evidence:** Art direction and continuity gap. Roads, camp, aid and civic encounters now differ, and the red-cap persona is no longer the default subject everywhere. However, encounter families reuse fixed illustrations without region or roster inputs. Camp is always a briefing table, the final vote reuses a civic podium scene, and the failure scene is a darkened road. Portraits and the van use stronger pixel contours than several softly shaded encounter figures.

**Why it matters:** Art selection can be technically valid while the scene still fails to communicate rest, arrival, a specific decision or a changed party. Reused character likenesses blur crew and opposition.

**Change:** Prioritize new arrival, recovery and terminal compositions. Maintain a scene register covering narrative job, route biome, time, weather, active subjects and outcome variant. Match the approved chunky pixel scale, palette, outlines and anatomy; review each asset inside the actual HUD crop at desktop and phone sizes.

**Acceptance:** A reviewer can identify the stage and main action with the title hidden. Every featured crew member is eligible, the region is plausible, and the scene survives both viewport crops without anatomy or style drift.

**Owner:** Art direction, asset register and scene selection.

**Screens:** [02-persona](screenshots/02-persona.png), [08-transit](screenshots/08-transit.png), [12-camp-arrival](screenshots/12-camp-arrival.png), [20-absent-crew-encounter](screenshots/20-absent-crew-encounter.png), [27-final-vote](screenshots/27-final-vote.png), [24-failure-ending](screenshots/24-failure-ending.png).

**Source:** [dystrail-web/src/components/ui/journey_scene/mod.rs:45](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/journey_scene/mod.rs:45), [dystrail-web/src/components/ui/journey_scene/encounters.rs:1](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/journey_scene/encounters.rs:1), [dystrail-web/src/components/ui/world_view.rs:41](/Users/vanna/Source/Dystrail/dystrail-web/src/components/ui/world_view.rs:41).

## Proposed play loop

**Prepare → Depart → sustained road travel → meaningful interruption → decision and consequence → Resume.**

**Approaching town → local arrival scene → shop / trade / talk / recover → deliberate departure.**

**A safe periodic checkpoint → full map scene → trace progress and anticipate the next leg → Resume.**

The map remains a periodic scene. Inventory, journal and current setting comparisons belong to the player-requested assessment state. Show the selected pace and information diet during travel; when assessing the situation, display the alternatives and their effects together so the player does not click to consume basic information.

| Event kind | Presentation | Player control |
|---|---|---|
| Ambient road detail | Brief scene change or line of text; journal if useful | Travel continues; pause remains available |
| Material choice or critical condition | Stop travel; show cause, named participants, costs and uncertainty | Explicit choice; no timer expires a decision |
| Routine weather change | Visible road atmosphere and concise effect | Continue if manageable; stop when the change creates a material risk |
| Arrival | Local establishing scene and services | Shop, trade, talk, rest, or explicitly depart |
| Map checkpoint | Full map scene at a safe break | Review next towns; Resume continues the trip |
| Crew loss or terminal event | Distinct, readable consequence scene | Time to absorb it; outcome persists through recovery |

The moving van should be the default state of an ongoing leg. The meaningful tension is whether to intervene before conditions worsen, not how long a fixed progress bar takes. For accessibility, keep a clear pause control, optional step mode, reduced motion, persistent decisions, and automatic pause when the tab is hidden. Speed controls govern presentation; they must not skip consequences or multiply state changes.

## Tone and satire

The strongest current material makes institutional absurdity concrete. The Department of Renaming Things has a premise that can affect a route, permit or time cost. The community exchange's subscription-economy joke sits beside a real exchange. Civic Center Potluck provides relief and human solidarity. Keep that range.

The weaker pattern is a generic sardonic sentence attached to unexplained stat arithmetic. Repeated Live with it. headers make a helpful meal, needed rest and catastrophe feel emotionally interchangeable. Let communities be capable and generous; let bureaucracies, incentives and platforms create the absurd constraint. Crew banter can carry personality without every speaker delivering the same cynical voice.

These are **fictional design examples**, not claims about current policies:

| Place and situation | Satirical mechanism | Meaningful choice and payoff |
|---|---|---|
| A river crossing near La Crosse | A private operator sells a Premium Queue subscription | Pay cash for speed, take a known longer detour, or use documented access; show actual distance/time/resource effects |
| A cooling center during a heat wave | Public funding vanished, but the donation platform still has a processing fee | Spend time helping volunteers, contribute stock, or keep driving with explicit exposure risk; a named crew member responds |
| A permit office on the approach to D.C. | The agency has renamed the form twice since you printed it | Use a relevant receipt, ask a capable crew member to negotiate, or wait; prior choices unlock a different approach |

Use regional stereotypes as jokes about local industries, civic branding, road culture and institutional incentives. Anchor them in the actual route. The map's marginal satire should never obscure a real town or relocate geography to serve a joke. Classic should retain readable stakes and room for solidarity; The Deep End can escalate absurdity and volatility while preserving the same comprehensible interaction rules.

## Art continuity contract

| Scene | Job | Required continuity |
|---|---|---|
| Open road | Convey distance, changing landscape and time | Normal van; three rows of two; active roster only; regional terrain; correct direction; restrained vehicle motion |
| Encounter | Make the specific situation legible before the text | Eligible featured persona; appropriate local setting; natural anatomy; weather indoors/outdoors handled correctly |
| Town arrival | Reward progress and reveal local opportunities | Recognizable local cue, map-aligned town, shop / aid / trade identity and next-leg context |
| Camp / recovery | Communicate slowing down and caring for the crew | Rest activity rather than another briefing; eligible cast; persisted time; visible repair or recovery state |
| Map | Orient and anticipate | Accurate geography; persona route; current position; next towns; satire outside critical labels |
| Outcome | Show what changed | Same people and place as the triggering event; appropriate emotion and actual resource consequence |
| Ending | Close this crew's particular trip | Active / absent / lost member distinctions, reached location, decisive moments, clear reason for ending |

Use the existing approved portraits and normal van as style anchors: chunky pixel contours, restrained palette, simplified volumes and readable expressions. Several encounters have softer painterly shading; reconcile them to those anchors before making more batches. Avoid photorealistic machinery, stretched vehicles and anatomy that only works at thumbnail size. Every art review should inspect the actual desktop and phone crop with the HUD and title in place. Label recognition is not enough: hide the title and confirm that the stage and action still read.

The next art batch should prioritize **town arrivals, actual recovery, named crew incidents and distinct endings**. A growing pool of general road or civic images will not solve those missing narrative jobs. Scene selection should use stage, location, time, weather, eligible cast and consequence; the underlying route and party state remain authoritative.

## Keep from the current pass

- The full name and D.C. wording are present throughout the sampled screens.
- The geographically grounded map has route zoom, upcoming services and satire in its margins. Road art changes between prairie and river/orchard terrain on the sampled route.
- The van has three normal seat rows and six visible crew members. Controlled absent-member states remove them and fill the front seats from the remaining crew.
- The top menu groups Save, Load, High contrast and Language. Both the menu and tooltip dismissed on outside clicks in the checks performed.
- The phone travel view had no horizontal overflow; its primary travel button was visible. The weather tooltip stayed on screen at 390 × 844.
- A numbered encounter choice worked, transactions showed actual costs, and the tested terminal state auto-resumed after reload. No browser errors were reported in the inspected session.

## Suggested sequence and next-round checks

1. **Repair trust:** stage scroll entry; truthful effective previews; latest receipt and journal persistence; clock/rest/weather consistency; persona origin contrast. These are bounded fixes with direct reproductions.
2. **Restore travel rhythm:** continuous legs, player pause, interruption priority and honest progression. Keep the periodic full map scene. Validate with a complete first leg, not just one tick.
3. **Make places and people matter:** town arrival mode and art, locally useful services and conversation, named crew effects, roster-aware scene casting. Connect persona origin to understandable first-leg preparation.
4. **Pay off the trip:** establish the D.C. objective early, show how travel decisions shape the final challenge, and finish with a crew-centered recap. Then tighten visual style and language consistency across every stage.

For the next playthrough, use a natural first leg from onboarding through a shop, a weather incident, a named crew event, a rest and an arrival. Reload at an unresolved encounter and immediately after a consequence. Ask the player what changed, why it changed, who was affected, where they are, and what they are watching for next. Then run controlled edge cases for caps, absence, danger and endings. The test succeeds when those answers come from the game itself, without an explanation from its maker.

Do not treat scene-file coverage or a green automated test suite as proof of this experience. This review found valid images, working buttons and correct clamping alongside misleading outcomes and broken narrative continuity. The acceptance checks above evaluate those meanings directly.
