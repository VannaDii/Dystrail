# Dystopian Trail — scene and route associations

Each selected persona has a persisted origin and an actual driving route to D.C. Census state boundaries and OpenStreetMap road geometry share an Albers projection. The map includes Alaska and Hawaii as insets; its traced route and van marker follow the selected roads. [Source data, attribution and reproducible generation](map-sources/README.md).

| Persona | Starting city |
| --- | --- |
| Journalist | Minneapolis, Minnesota |
| Organizer | Kansas City, Missouri |
| Whistleblower | Denver, Colorado |
| Lobbyist | St. Louis, Missouri |
| Staffer | Omaha, Nebraska |
| Satirist | Austin, Texas |

The six routes join at Chicago and continue through South Bend, Toledo, Cleveland, Pittsburgh, Cumberland, Hagerstown, Frederick and D.C. Earlier legs contain real persona-specific towns. Supply shops and community trading become available when travel crosses one of these towns.

| Geographic leg | Engine region | Road art |
| --- | --- | --- |
| Origins and plains | Heartland | Prairie farmland |
| La Crosse / Madison / Des Moines / Iowa City | Heartland | Orchards and river country |
| Chicago / South Bend | Rust Belt | Foundry district |
| Toledo / Cleveland | Rust Belt | Lakeside industry |
| Pittsburgh / Cumberland / Hagerstown | Rust Belt | Appalachian wooded ridges |
| Frederick toward D.C. | Beltway | Suburban office corridor |
| D.C. arrival | Beltway | Parkway |

The map is a **full scene after the travel animation**, first shown on the initial journey action, then at new towns, region changes, and five-day checkpoints. It replaces the road and HUD instead of adding a second component. Continue reveals the already-resolved next decision. Both the pending map and its dismissal survive recovery. Whole-U.S. and route-detail views use the same actual geographic geometry. A concise list names the next two towns, remaining road miles and services. Regional satire appears as pixel icons/labels and a persona-specific travel caption, without moving towns or distorting state boundaries.

Physical mileage is proportional to the existing strategy's simulation distance budget; player-visible mileage and geography consistently use the physical route. Region is synchronized on travel, rollback and older-save migration. Changing the day alone cannot switch to another geographic scene.

| Stage | Scene / presentation |
| --- | --- |
| Setup | Prairie road with the six-person crew in a normal van |
| Persona | Six-person portrait atlas |
| Crew naming | Player name, crew name, five editable generated companion names |
| Outfitting / revisiting shop | Catalog, cart and contextual equipment help |
| Travel | Route-specific open road plus independently rendered van and crew |
| Map interlude | Full geographic U.S. map, traced route, upcoming towns and regional satire |
| In transit | Van drives across that road; time and weather layers update; skip is available |
| Encounter | Explicit encounter setting from the table below, with current town or route-leg caption |
| Breakdown | Service scene and repair resource receipt |
| Camp | Night briefing scene, actual region/location, rest and repair controls |
| Town stop | Current road/location, named supply store and community exchange |
| Aftermath | Triggering event's scene and resource changes |
| Final confrontation | Beltway civic scene |
| Result | Regional art with outcome treatment and persona portrait |

## Encounter families

All 35 shipped IDs have explicit assignments. These are shared illustrations for related settings, not 35 unique illustrations. Unknown IDs do not silently inherit generic road art.

| Illustration | Encounters | Primary crew subject |
| --- | --- | --- |
| Media workshop | `classic_media_training`, `deep_media_ambush` | Staffer, with Whistleblower observing |
| Community aid | `classic_civic_potluck`, `classic_mutual_aid`, `classic_water_drive`, `fundraiser_detour` | Organizer |
| Bridge crew | `classic_bridge_crews` | Organizer directing Staffer and Satirist |
| Vehicle service | `classic_service_station`, `deep_circuit_breaker` | Staffer |
| Civic briefing | `beltway_briefing`, `classic_press_briefing`, `classic_press_pool_qna`, `deep_state_dirge`, `deep_watchdog_sync`, `town_hall_drift` | Lobbyist and Journalist |
| Checkpoint | `tariff_whiplash`, `deep_beltway_fastpass` | Lobbyist |
| Clinic | `clinic_triage` | Journalist |
| Radio / coordination | `classic_mail_drop`, `classic_mutual_aid_dispatch`, `classic_neighborhood_watch`, `classic_radio_phonebank`, `deep_field_intel`, `deep_grassroots_signal`, `deep_memorandum_dump`, `deep_secure_line` | Whistleblower and Organizer |
| Street gathering | `classic_crossing_block_party`, `classic_freeway_mural`, `classic_overpass_stage`, `classic_union_blockade` | Satirist |
| Convoy | `deep_waystation_boost`, `deep_rustbelt_convoy` | Staffer and Organizer |
| Night briefing | `deep_watch_party`, `overnight_briefing` | Whistleblower, Journalist, Organizer |
| Raw milk | `raw_milk` | Existing mode-specific dairy scene |

Outdoor scenes receive actual rain, snow, heat shimmer, smoke, and time-of-day treatments. Indoor workshop/civic/radio scenes avoid indoor precipitation. Night briefings use 22:00, camp uses 20:00, and the Deep dairy scene uses 18:00 to match its painted sunset; the HUD and light share one clock value. Reduced motion removes travel/weather animation while retaining static conditions.

## Dynamic van

The van has three rows of two seats. The empty body and six transparent persona busts are independent layers. Active status controls which busts render; deceased and departed members keep their names/status in the roster but disappear from the van. An available member takes the wheel if the usual driver leaves. Names and presence survive manual saves and automatic recovery.

The full-crew van's alpha silhouette masks the empty body because the generated empty-body asset contains a baked checkerboard outside the vehicle. Only the alpha is used: its painted occupants are never displayed. No stretched six-window van is shipped as the active vehicle. Event illustrations remain authored ensembles; individual character layers apply to road/travel van occupants.
